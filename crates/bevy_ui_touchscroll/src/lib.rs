//! Touch and wheel scrolling for Bevy UI `ScrollArea`s that feels native on phones.
//!
//! - A touch drag scrolls the nearest scroll area along its main axis 1:1 with the finger
//!   (mouse drags are left alone, e.g. for text selection).
//! - On release, the page flings with Android's physics: Flutter's `VelocityTracker`
//!   estimates the release velocity, and `ClampingScrollSimulation`'s curve decelerates it.
//!   A finger landing on a fling stops it.
//! - A touch that scrolls (it moved past [`TOUCH_SLOP`] over a scroll area) or stops a fling
//!   is not a tap: buttons it pressed are released without activating, and other click
//!   handlers check [`TouchScroll::gesture_scrolled`].
//! - Wheel, keyboard and "scroll into view" jumps are eased ([`SmoothScroll`]), and the wheel
//!   distance is scaled by [`ScrollSpeed`].
//! - Pressing the mouse wheel starts autoscroll, as in browsers: the page scrolls towards the
//!   pointer, faster the further it is from where the wheel was pressed. Holding the wheel
//!   scrolls until it is let go; a quick click scrolls until the next press (or `Escape`).
//!   [`TouchScroll::autoscroll`] tells where to draw its marker.

use bevy::{
    picking::events::{PointerDrag, PointerDragEnd, PointerDragStart},
    prelude::*,
};
use bevy_ui_textselect::TextSelection;

/// Adds touch scrolling, flings and smooth wheel scrolling to every `ScrollArea`.
pub struct TouchScrollPlugin;

impl Plugin for TouchScrollPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchScroll>()
            .init_resource::<ScrollSpeed>()
            .add_observer(touch_drag_start)
            .add_observer(touch_drag)
            .add_observer(touch_drag_end)
            .add_observer(on_press)
            .add_observer(add_smooth_scroll)
            .add_observer(boost_wheel)
            .add_systems(
                Update,
                (touch_momentum, autoscroll, release_scrolled_buttons),
            )
            .add_systems(
                PostUpdate,
                smooth_scroll.before(bevy::ui::UiSystems::Layout),
            );
    }
}

/// Scales wheel distance and fling speed (1.0 = the platform's distance).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ScrollSpeed(pub f32);

impl Default for ScrollSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

/// State of the touch drag or fling in progress.
#[derive(Resource, Default)]
pub struct TouchScroll {
    /// Touch drag that has not picked a scroll area yet (decided by its first movement).
    pending: Option<(Entity, bevy::picking::pointer::PointerId)>,
    /// Scroll area being dragged, and the pointer dragging it.
    active: Option<(Entity, bevy::picking::pointer::PointerId)>,
    /// Scroll area coasting after release.
    coasting: Option<Entity>,
    /// The fling it follows.
    fling: Option<Fling>,
    /// Recent finger positions, for the release velocity.
    tracker: VelocityTracker,
    /// The current (or just ended) press scrolled or stopped a fling: it is not a tap.
    scrolled: bool,
    /// Recent frame lengths, for even fling steps.
    frames: FrameClock,
    /// Time on the fling's curve shown this frame.
    fling_time: f64,
    /// Middle-button autoscroll in progress.
    autoscroll: Option<Autoscroll>,
}

/// A middle-button autoscroll.
#[derive(Clone, Copy, Debug)]
struct Autoscroll {
    target: Entity,
    /// Where the wheel was pressed (logical window px).
    origin: Vec2,
    /// The axes the target scrolls along.
    axes: BVec2,
    /// When the wheel was pressed (real seconds).
    pressed_at: f64,
    /// The wheel is still held down.
    holding: bool,
    /// The pointer left the dead zone while the wheel was held: letting go ends the scroll.
    moved: bool,
}

/// Distance (logical px) from the autoscroll's origin within which nothing scrolls.
pub const AUTOSCROLL_DEAD_ZONE: f32 = 12.0;
/// A wheel press shorter than this (s), without moving, keeps autoscrolling once let go.
const AUTOSCROLL_CLICK: f64 = 0.3;

/// Autoscroll speed (px/s) at `distance` px past the dead zone: slow near it, quick far out.
fn autoscroll_speed(distance: f32) -> f32 {
    distance * 5.0 + distance * distance * 0.03
}

impl TouchScroll {
    /// Whether the current press (or the one that just ended) scrolled or stopped a fling.
    /// Its release is then not a tap: a click it ends should do nothing. Buttons it pressed
    /// are released without activating; other click handlers check this.
    pub fn gesture_scrolled(&self) -> bool {
        self.scrolled
    }

    /// Whether a finger is scrolling, a fling is coasting or autoscroll runs (frames are
    /// needed).
    pub fn busy(&self) -> bool {
        self.pending.is_some()
            || self.active.is_some()
            || self.coasting.is_some()
            || self.autoscroll.is_some()
    }

    /// The running autoscroll: where the wheel was pressed (logical window px) and the axes
    /// it scrolls, for drawing its marker.
    pub fn autoscroll(&self) -> Option<(Vec2, BVec2)> {
        self.autoscroll.map(|a| (a.origin, a.axes))
    }
}

// Fling physics, as on Android (Flutter's `ClampingScrollSimulation`, which matches
// Android's `OverScroller`): the page decelerates along a fixed curve whose length and
// distance follow from the release velocity.
const DECELERATION_RATE: f64 = 2.358_201_815; // ln(0.78) / ln(0.9)
const INFLEXION: f64 = 0.35;
const FRICTION: f64 = 0.015;
/// Earth's gravity in logical px/s² at 160 dpi, times 0.84 (Android's physical coefficient).
const PHYSICAL_COEFF: f64 = 9.806_65 * 39.37 * 160.0 * 0.84;
/// Slower releases do not fling; faster ones are capped (Flutter's limits, px/s).
const MIN_FLING_VELOCITY: f32 = 50.0;
const MAX_FLING_VELOCITY: f32 = 8000.0;

#[derive(Clone, Copy, Debug)]
struct Fling {
    start_time: f64,
    start: Vec2,
    /// Total travel (px).
    distance: Vec2,
    /// Length (s).
    duration: f64,
}

impl Fling {
    fn new(now: f64, start: Vec2, velocity: Vec2) -> Option<Self> {
        let speed = f64::from(velocity.length());
        if speed < f64::from(MIN_FLING_VELOCITY) {
            return None;
        }
        let reference = FRICTION * PHYSICAL_COEFF / INFLEXION;
        let duration = DECELERATION_RATE
            * INFLEXION
            * (speed / reference).powf(1.0 / (DECELERATION_RATE - 1.0));
        Some(Self {
            start_time: now,
            start,
            distance: velocity * (duration / DECELERATION_RATE) as f32,
            duration,
        })
    }

    /// Position at `now`, and whether the fling is over.
    fn at(&self, now: f64) -> (Vec2, bool) {
        let t = ((now - self.start_time) / self.duration).clamp(0.0, 1.0);
        let progress = 1.0 - (1.0 - t).powf(DECELERATION_RATE);
        (self.start + self.distance * progress as f32, t >= 1.0)
    }
}

/// Display frame length, estimated from recent updates. A fling advances by whole frames of
/// this length rather than by the measured time between updates: that time wobbles by a few
/// milliseconds, which at fling speeds is a visible judder.
#[derive(Default)]
struct FrameClock {
    /// Recent frame lengths (s), newest last.
    deltas: std::collections::VecDeque<f64>,
}

impl FrameClock {
    const WINDOW: usize = 15;

    fn record(&mut self, delta: f64) {
        // Longer gaps are idle waits, not display frames.
        if delta <= 0.0 || delta > 0.05 {
            return;
        }
        if self.deltas.len() == Self::WINDOW {
            self.deltas.pop_front();
        }
        self.deltas.push_back(delta);
    }

    /// Median recent frame length.
    fn interval(&self) -> Option<f64> {
        let mut sorted: Vec<f64> = self.deltas.iter().copied().collect();
        sorted.sort_by(f64::total_cmp);
        sorted.get(sorted.len() / 2).copied()
    }

    /// Advances `clock` by the whole frames that fit in `delta`, staying within a frame of
    /// `now`.
    fn advance(&self, clock: f64, delta: f64, now: f64) -> f64 {
        let Some(frame) = self.interval() else {
            return now;
        };
        let frames = (delta / frame).round().max(1.0);
        (clock + frames * frame).clamp(now - frame, now + frame)
    }
}

/// Release velocity from recent finger positions: a least-squares quadratic fit over the
/// last 100 ms, as Flutter's `VelocityTracker` does (a finger resting for 40 ms means zero).
#[derive(Default)]
struct VelocityTracker {
    /// (time in s, total finger travel in scroll px).
    samples: std::collections::VecDeque<(f64, Vec2)>,
    travel: Vec2,
}

impl VelocityTracker {
    const HORIZON: f64 = 0.100;
    const STOPPED: f64 = 0.040;

    fn reset(&mut self) {
        self.samples.clear();
        self.travel = Vec2::ZERO;
    }

    fn add(&mut self, now: f64, delta: Vec2) {
        self.travel += delta;
        // Several moves can arrive in one frame: keep one sample per timestamp.
        if let Some(last) = self.samples.back_mut()
            && (now - last.0).abs() < 1e-6
        {
            last.1 = self.travel;
        } else {
            self.samples.push_back((now, self.travel));
        }
        while self.samples.len() > 20 {
            self.samples.pop_front();
        }
    }

    /// Velocity at `now`. Samples are stamped with frame times, so pauses shorter than two
    /// frames (`frame` s each) are not taken as the finger resting.
    fn velocity(&self, now: f64, frame: f64) -> Vec2 {
        let Some(&(newest, _)) = self.samples.back() else {
            return Vec2::ZERO;
        };
        let stopped = Self::STOPPED.max(2.0 * frame);
        if now - newest > stopped {
            return Vec2::ZERO;
        }
        // Samples within the horizon, without a pause between them.
        let horizon = Self::HORIZON.max(3.0 * frame);
        let mut used: Vec<(f64, Vec2)> = Vec::new();
        let mut previous = newest;
        for &(time, travel) in self.samples.iter().rev() {
            if newest - time > horizon || previous - time > stopped {
                break;
            }
            used.push((time - newest, travel));
            previous = time;
        }
        match used.len() {
            0 | 1 => Vec2::ZERO,
            2 => {
                let ((t0, p0), (t1, p1)) = (used[1], used[0]);
                if t1 - t0 <= 0.0 {
                    Vec2::ZERO
                } else {
                    (p1 - p0) / (t1 - t0) as f32
                }
            }
            _ => Vec2::new(
                quadratic_slope(&used, |p| p.x),
                quadratic_slope(&used, |p| p.y),
            ),
        }
    }
}

/// Least-squares fit of `y = a + b·t + c·t²`; returns `b`, the slope at `t = 0` (the newest
/// sample).
fn quadratic_slope(samples: &[(f64, Vec2)], axis: impl Fn(Vec2) -> f32) -> f32 {
    // Normal equations: sums of t^k and y·t^k.
    let mut s = [0.0_f64; 5];
    let mut r = [0.0_f64; 3];
    for &(t, p) in samples {
        let y = f64::from(axis(p));
        let mut tk = 1.0;
        for (k, sk) in s.iter_mut().enumerate() {
            *sk += tk;
            if k < 3 {
                r[k] += y * tk;
            }
            tk *= t;
        }
    }
    let m = [[s[0], s[1], s[2]], [s[1], s[2], s[3]], [s[2], s[3], s[4]]];
    let det = |m: [[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(m);
    if d.abs() < 1e-18 {
        return 0.0;
    }
    // Cramer's rule for the middle coefficient.
    let mut mb = m;
    for (row, value) in mb.iter_mut().zip(r) {
        row[1] = value;
    }
    (det(mb) / d) as f32
}

/// The nearest ancestor (or self) of `entity` that can scroll along the drag's main axis,
/// so a vertical swipe over a sideways-scrolling code block still scrolls the page.
fn scroll_target(
    entity: Entity,
    delta: Vec2,
    parents: &Query<&ChildOf>,
    areas: &Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
) -> Option<Entity> {
    let vertical = delta.y.abs() >= delta.x.abs();
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find(|e| {
            areas.get(*e).is_ok_and(|(node, computed, _)| {
                let overflow = computed.content_size() - computed.size();
                if vertical {
                    node.overflow.y == OverflowAxis::Scroll && overflow.y > 1.0
                } else {
                    node.overflow.x == OverflowAxis::Scroll && overflow.x > 1.0
                }
            })
        })
}

/// Drags that start on this entity or inside it are its own, not scrolls (e.g. a game view
/// whose drags move things).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct OwnsDrags;

fn touch_drag_start(
    drag: On<PointerDragStart>,
    parents: Query<&ChildOf>,
    owners: Query<(), With<OwnsDrags>>,
    mut scroll: ResMut<TouchScroll>,
) {
    if !drag.pointer.id.is_touch() || drag.entity != drag.original_event_target() {
        return;
    }
    if std::iter::once(drag.entity)
        .chain(parents.iter_ancestors(drag.entity))
        .any(|entity| owners.contains(entity))
    {
        return;
    }
    scroll.pending = Some((drag.entity, drag.pointer.id));
    scroll.active = None;
    scroll.coasting = None;
    scroll.fling = None;
    scroll.tracker.reset();
}

/// Distance (logical px) a finger moves before its drag scrolls: shorter moves are a shaky
/// tap (Android's touch slop).
pub const TOUCH_SLOP: f32 = 8.0;

/// A new press starts a gesture that has not scrolled. A finger landing on a fling stops it,
/// like on Android; that touch is not a tap either. A press during autoscroll only ends it;
/// a wheel press over a scroll area starts it.
fn on_press(
    press: On<bevy::picking::events::PointerPress>,
    time: Res<Time<Real>>,
    parents: Query<&ChildOf>,
    areas: Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
    mut scroll: ResMut<TouchScroll>,
) {
    if press.entity != press.original_event_target() {
        return;
    }
    scroll.scrolled = false;
    if scroll.autoscroll.take().is_some() {
        scroll.scrolled = true;
        return;
    }
    if press.pointer.id.is_touch() && scroll.coasting.take().is_some() {
        scroll.fling = None;
        scroll.scrolled = true;
    }
    if press.button != bevy::picking::pointer::PointerButton::Middle || press.pointer.id.is_touch()
    {
        return;
    }
    // The nearest area that scrolls down, else sideways.
    let Some(target) = scroll_target(press.entity, Vec2::Y, &parents, &areas)
        .or_else(|| scroll_target(press.entity, Vec2::X, &parents, &areas))
    else {
        return;
    };
    let Ok((node, computed, _)) = areas.get(target) else {
        return;
    };
    let overflow = computed.content_size() - computed.size();
    scroll.autoscroll = Some(Autoscroll {
        target,
        origin: press.pointer.position,
        axes: BVec2::new(
            node.overflow.x == OverflowAxis::Scroll && overflow.x > 1.0,
            node.overflow.y == OverflowAxis::Scroll && overflow.y > 1.0,
        ),
        pressed_at: time.elapsed_secs_f64(),
        holding: true,
        moved: false,
    });
    scroll.coasting = None;
    scroll.fling = None;
    // Not a click: a button under the wheel lets go without activating.
    scroll.scrolled = true;
}

/// Scrolls the autoscroll's area towards the pointer, and ends it when the held wheel is let
/// go (after moving or holding it), on `Escape`, or when the area goes away.
fn autoscroll(
    time: Res<Time<Real>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    speed: Res<ScrollSpeed>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut scroll: ResMut<TouchScroll>,
    mut areas: Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
) {
    let Some(mut auto) = scroll.autoscroll else {
        return;
    };
    if keys.just_pressed(KeyCode::Escape) || !areas.contains(auto.target) {
        scroll.autoscroll = None;
        return;
    }
    let offset = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .map_or(Vec2::ZERO, |cursor| cursor - auto.origin);
    if offset.length() > AUTOSCROLL_DEAD_ZONE {
        auto.moved = true;
    }
    if auto.holding && !mouse.pressed(MouseButton::Middle) {
        auto.holding = false;
        let click = !auto.moved && time.elapsed_secs_f64() - auto.pressed_at < AUTOSCROLL_CLICK;
        if !click {
            scroll.autoscroll = None;
            return;
        }
    }
    let axis = |d: f32, on: bool| {
        if on {
            d.signum() * autoscroll_speed((d.abs() - AUTOSCROLL_DEAD_ZONE).max(0.0))
        } else {
            0.0
        }
    };
    let velocity = Vec2::new(axis(offset.x, auto.axes.x), axis(offset.y, auto.axes.y));
    apply_scroll(
        auto.target,
        velocity * speed.0 * time.delta_secs().min(0.1),
        &mut areas,
    );
    scroll.autoscroll = Some(auto);
}

/// Buttons pressed by a touch that turned into a scroll let go without activating (Bevy's
/// buttons only activate on a click while pressed).
fn release_scrolled_buttons(
    scroll: Res<TouchScroll>,
    pressed: Query<Entity, With<bevy::ui::Pressed>>,
    mut commands: Commands,
) {
    if scroll.scrolled {
        for button in &pressed {
            commands.entity(button).remove::<bevy::ui::Pressed>();
        }
    }
}

fn touch_drag(
    drag: On<PointerDrag>,
    time: Res<Time<Real>>,
    selection: Res<TextSelection>,
    parents: Query<&ChildOf>,
    mut scroll: ResMut<TouchScroll>,
    mut areas: Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
) {
    // The finger must move past the slop before the drag scrolls (and stops being a tap).
    if let Some((entity, pointer)) = scroll.pending
        && pointer == drag.pointer.id
        && drag.distance.length() >= TOUCH_SLOP
    {
        scroll.pending = None;
        scroll.active =
            scroll_target(entity, drag.distance, &parents, &areas).map(|e| (e, pointer));
        if scroll.active.is_some() {
            scroll.scrolled = true;
        }
    }
    let Some((target, pointer)) = scroll.active else {
        return;
    };
    if drag.pointer.id != pointer || drag.entity != drag.original_event_target() {
        return;
    }
    // A long-press selection owns the finger.
    if selection.is_touch_selecting() {
        scroll.active = None;
        return;
    }
    apply_scroll(target, -drag.delta, &mut areas);
    scroll.tracker.add(time.elapsed_secs_f64(), -drag.delta);
}

fn touch_drag_end(
    drag: On<PointerDragEnd>,
    time: Res<Time<Real>>,
    speed: Res<ScrollSpeed>,
    areas: Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
    mut scroll: ResMut<TouchScroll>,
) {
    scroll.pending = None;
    if let Some((target, pointer)) = scroll.active
        && pointer == drag.pointer.id
    {
        scroll.active = None;
        let now = time.elapsed_secs_f64();
        // The finger moves the page 1:1; the scroll speed setting scales the fling.
        let frame = f64::from(time.delta_secs()).min(0.1);
        let velocity =
            (scroll.tracker.velocity(now, frame) * speed.0).clamp_length_max(MAX_FLING_VELOCITY);
        let start = areas.get(target).map_or(Vec2::ZERO, |(_, _, p)| p.0);
        scroll.fling = Fling::new(now, start, velocity);
        scroll.fling_time = now;
        scroll.coasting = scroll.fling.is_some().then_some(target);
    }
}

/// Adds `delta` (logical px) to a scroll position, clamped. Returns whether it moved.
fn apply_scroll(
    target: Entity,
    delta: Vec2,
    areas: &mut Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
) -> bool {
    let Ok((node, computed, mut position)) = areas.get_mut(target) else {
        return false;
    };
    let max = ((computed.content_size() - computed.size()) * computed.inverse_scale_factor)
        .max(Vec2::ZERO);
    let mut next = position.0;
    if node.overflow.x == OverflowAxis::Scroll {
        next.x = (next.x + delta.x).clamp(0.0, max.x);
    }
    if node.overflow.y == OverflowAxis::Scroll {
        next.y = (next.y + delta.y).clamp(0.0, max.y);
    }
    if next == position.0 {
        return false;
    }
    position.0 = next;
    true
}

fn touch_momentum(
    time: Res<Time<Real>>,
    mut scroll: ResMut<TouchScroll>,
    mut areas: Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
) {
    let delta = time.delta_secs_f64();
    scroll.frames.record(delta);
    let (Some(target), Some(fling)) = (scroll.coasting, scroll.fling) else {
        return;
    };
    let now = time.elapsed_secs_f64();
    // The first frame can run in the same update as the release: it stays at the start.
    if now > fling.start_time {
        scroll.fling_time = scroll.frames.advance(scroll.fling_time, delta, now);
    }
    let (position, done) = fling.at(scroll.fling_time);
    let Ok((_, _, current)) = areas.get(target) else {
        scroll.coasting = None;
        return;
    };
    let step = position - current.0;
    let moved = apply_scroll(target, step, &mut areas);
    // Stop at the end of the curve, or at an edge (a step that could not move). The first
    // frame can run in the same update as the release, with no time elapsed and no step.
    if done || (step != Vec2::ZERO && !moved) {
        scroll.coasting = None;
        scroll.fling = None;
    }
}

// ---------------------------------------------------------------------------
// Smooth scrolling
// ---------------------------------------------------------------------------

/// Eases a scroll area towards the position that wheel, keyboard and "scroll into view"
/// requests set, instead of jumping there. Touch drags stay 1:1 with the finger.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SmoothScroll {
    /// Position currently shown.
    shown: Vec2,
    /// Position being eased towards.
    target: Vec2,
    /// Extra wheel distance from the scroll speed setting, added on the next frame.
    boost: Vec2,
}

impl SmoothScroll {
    /// Whether it is still easing towards its target.
    pub fn animating(&self) -> bool {
        self.shown.distance_squared(self.target) > 0.25
    }

    /// Jumps to `position` without easing (e.g. when a new note opens).
    pub fn snap(&mut self, position: Vec2) {
        self.shown = position;
        self.target = position;
    }
}

fn add_smooth_scroll(
    add: On<Add<bevy::ui_widgets::ScrollArea>>,
    positions: Query<&ScrollPosition>,
    mut commands: Commands,
) {
    let position = positions.get(add.entity).map_or(Vec2::ZERO, |p| p.0);
    commands.entity(add.entity).insert(SmoothScroll {
        shown: position,
        target: position,
        boost: Vec2::ZERO,
    });
}

/// The wheel scrolls `ScrollArea`s by the platform's distance; this adds the difference to
/// the scroll speed setting (applied in [`smooth_scroll`]).
fn boost_wheel(
    scroll: On<bevy::picking::events::PointerScroll>,
    speed: Res<ScrollSpeed>,
    pixels_per_line: Res<bevy::input::mouse::MouseScrollPixelsPerLine>,
    mut areas: Query<(&Node, &mut SmoothScroll), With<bevy::ui_widgets::ScrollArea>>,
) {
    let Ok((node, mut smooth)) = areas.get_mut(scroll.entity) else {
        return;
    };
    let delta = scroll.to_pixels(&pixels_per_line);
    let mut extra = -Vec2::new(delta.x, delta.y) * (speed.0 - 1.0);
    if node.overflow.x != OverflowAxis::Scroll {
        extra.x = 0.0;
    }
    if node.overflow.y != OverflowAxis::Scroll {
        extra.y = 0.0;
    }
    smooth.boost += extra;
}

fn smooth_scroll(
    time: Res<Time<Real>>,
    touch: Res<TouchScroll>,
    mut areas: Query<(
        Entity,
        &ComputedNode,
        &mut ScrollPosition,
        &mut SmoothScroll,
    )>,
) {
    // Frame-rate independent exponential easing (~90% of the way in 0.12 s).
    let t = 1.0 - (-19.0 * time.delta_secs().min(0.1)).exp();
    for (entity, computed, mut position, mut smooth) in &mut areas {
        let touching = touch.active.is_some_and(|(e, _)| e == entity)
            || touch.coasting == Some(entity)
            || touch.autoscroll.is_some_and(|a| a.target == entity);
        if position.0 != smooth.shown {
            // Someone else moved it: that is the new target.
            if touching {
                smooth.snap(position.0);
                continue;
            }
            smooth.target = position.0;
        }
        if smooth.boost != Vec2::ZERO {
            let max = ((computed.content_size() - computed.size()) * computed.inverse_scale_factor)
                .max(Vec2::ZERO);
            smooth.target = (smooth.target + smooth.boost).clamp(Vec2::ZERO, max);
            smooth.boost = Vec2::ZERO;
        }
        if !smooth.animating() {
            if smooth.shown != smooth.target {
                smooth.shown = smooth.target;
                position.0 = smooth.target;
            }
            continue;
        }
        smooth.shown = smooth.shown.lerp(smooth.target, t);
        position.0 = smooth.shown;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scrolling_touch_releases_its_buttons() {
        let mut world = World::new();
        world.insert_resource(TouchScroll::default());
        let button = world.spawn(bevy::ui::Pressed).id();
        world.run_system_cached(release_scrolled_buttons).unwrap();
        assert!(
            world.get::<bevy::ui::Pressed>(button).is_some(),
            "a tap keeps it"
        );
        world.resource_mut::<TouchScroll>().scrolled = true;
        world.run_system_cached(release_scrolled_buttons).unwrap();
        assert!(
            world.get::<bevy::ui::Pressed>(button).is_none(),
            "a scroll lets go"
        );
    }

    #[test]
    fn fling_clock_steps_in_whole_frames() {
        let mut frames = FrameClock::default();
        for delta in [0.009, 0.008, 0.0085, 0.0075, 0.0083] {
            frames.record(delta);
        }
        frames.record(1.0); // idle wait: ignored
        assert!((frames.interval().unwrap() - 0.0083).abs() < 1e-9);
        // A late update still shows one frame later; a missed frame shows two.
        assert!((frames.advance(1.0, 0.0105, 1.0105) - 1.0083).abs() < 1e-9);
        assert!((frames.advance(1.0, 0.017, 1.017) - 1.0166).abs() < 1e-9);
        // Never drifts more than a frame from real time.
        assert!((frames.advance(1.0, 0.0083, 1.05) - (1.05 - 0.0083)).abs() < 1e-9);
    }

    #[test]
    fn velocity_of_a_steady_finger() {
        let mut tracker = VelocityTracker::default();
        // 1200 px/s downwards, sampled at 120 Hz.
        for i in 0..12 {
            tracker.add(f64::from(i) / 120.0, Vec2::new(0.0, 10.0));
        }
        let v = tracker.velocity(11.0 / 120.0, 1.0 / 120.0);
        assert!((v.y - 1200.0).abs() < 1.0, "{v}");
        assert!(v.x.abs() < 1e-3);
        // A finger that rested for more than 40 ms before lifting does not fling.
        assert_eq!(
            tracker.velocity(11.0 / 120.0 + 0.05, 1.0 / 120.0),
            Vec2::ZERO
        );
        // At 10 fps (slow frames), a release one frame after the last move still flings.
        let mut slow = VelocityTracker::default();
        for i in 0..4 {
            slow.add(f64::from(i) / 10.0, Vec2::new(0.0, 120.0));
        }
        let v = slow.velocity(0.4, 0.1);
        assert!((v.y - 1200.0).abs() < 1.0, "{v}");
    }

    #[test]
    fn fling_matches_android_curve() {
        let fling = Fling::new(0.0, Vec2::ZERO, Vec2::new(0.0, 3000.0)).unwrap();
        // Same numbers as Flutter's ClampingScrollSimulation for 3000 px/s.
        assert!((fling.duration - 1.028).abs() < 0.01, "{}", fling.duration);
        assert!(
            (fling.distance.y - 1308.0).abs() < 10.0,
            "{}",
            fling.distance
        );
        let (end, done) = fling.at(fling.duration);
        assert!(done && (end.y - fling.distance.y).abs() < 1e-3);
        // Starts at the release speed: ~3000 px/s over the first millisecond.
        let (early, _) = fling.at(0.001);
        assert!(
            (early.y / 0.001 - 3000.0).abs() < 30.0,
            "{}",
            early.y / 0.001
        );
        assert!(Fling::new(0.0, Vec2::ZERO, Vec2::new(0.0, 10.0)).is_none());
    }
}
