//! Responsive layout and touch support.
//!
//! - [`LayoutMode`] follows the window width: sidebars are inline on wide screens and become
//!   slide-over drawers on tablets and phones.
//! - Touch drags scroll the nearest `ScrollArea` 1:1, then fling with Android's physics
//!   (mouse drags select text).
//! - On touch devices, a selection action bar offers Copy / Select all, and focusing a text
//!   field shows an in-app keyboard (a Feathers [`VirtualKeyboard`]), since a WebGL canvas
//!   cannot summon the phone's keyboard.

use bevy::{
    feathers::controls::{VirtualKeyPressed, VirtualKeyboard},
    input::{mouse::MouseButtonInput, touch::TouchInput},
    input_focus::InputFocus,
    picking::events::{PointerDrag, PointerDragEnd, PointerDragStart},
    prelude::*,
    text::{EditableText, FontWeight, TextEdit},
    ui_widgets::Activate,
    window::PrimaryWindow,
};
use std::time::Duration;

use bevy_radioactive::coat::coat;
use bevy_ui_textselect::{ClearSelection, CopySelection, SelectAll, TextSelection};

use crate::{nav::Nav, ui::palette};

pub fn plugin(app: &mut App) {
    app.init_resource::<TouchScroll>()
        .init_resource::<KeyboardTarget>()
        .add_observer(touch_drag_start)
        .add_observer(touch_drag)
        .add_observer(touch_drag_end)
        .add_observer(catch_fling)
        .add_observer(add_smooth_scroll)
        .add_observer(boost_wheel)
        .add_systems(
            PostUpdate,
            smooth_scroll.before(bevy::ui::UiSystems::Layout),
        )
        .add_systems(
            Update,
            (
                update_layout_mode,
                detect_touch,
                touch_momentum,
                sync_selection_bar,
                track_keyboard_target,
            ),
        )
        .add_systems(Last, keep_awake);
}

/// The app runs in reactive mode (it only renders when something happens) to save battery and
/// keep phones cool. This requests extra frames while anything is animating or settling.
#[allow(clippy::too_many_arguments)]
fn keep_awake(
    scroll: Res<TouchScroll>,
    selection: Res<TextSelection>,
    graph: Res<crate::graph::GraphState>,
    nav: Res<Nav>,
    focus: Res<InputFocus>,
    editable: Query<(), With<EditableText>>,
    layout_changed: Query<(), Or<(Changed<ComputedNode>, Changed<ScrollPosition>, Added<Node>)>>,
    smooth: Query<&SmoothScroll>,
    mut winit: ResMut<bevy::winit::WinitSettings>,
    mut settle: Local<u8>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    let busy = scroll.pending.is_some()
        || scroll.active.is_some()
        || scroll.coasting.is_some()
        || selection.is_busy()
        || (nav.view.get_untracked() == crate::nav::View::Graph && graph.animating())
        || nav.anchor.with_untracked(Option::is_some)
        || focus.get().is_some_and(|e| editable.contains(e))
        || smooth.iter().any(SmoothScroll::animating);
    if busy || !layout_changed.is_empty() {
        // A few more frames let layout, text and reactive regions settle after a change.
        *settle = 3;
    }
    let cap = nav.max_fps.get_untracked();
    let mode = if *settle > 0 {
        *settle -= 1;
        if cap == 0 {
            // Next frame as soon as the display allows.
            redraw.write(bevy::window::RequestRedraw);
            IDLE
        } else {
            // Frame-rate cap: the next update comes from a timer, and input waits for it.
            bevy::winit::UpdateMode::Reactive {
                wait: Duration::from_secs_f64(1.0 / f64::from(cap)),
                react_to_device_events: false,
                react_to_user_events: false,
                react_to_window_events: false,
            }
        }
    } else {
        IDLE
    };
    if winit.focused_mode != mode {
        winit.focused_mode = mode;
    }
}

/// Update mode while nothing animates: wake on input, or once a second.
pub const IDLE: bevy::winit::UpdateMode = bevy::winit::UpdateMode::Reactive {
    wait: Duration::from_secs(1),
    react_to_device_events: true,
    react_to_user_events: true,
    react_to_window_events: true,
};

/// Width class of the window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutMode {
    /// Both sidebars inline (≥ 1100 px).
    #[default]
    Wide,
    /// Files inline, outline/backlinks as a drawer (≥ 760 px).
    Medium,
    /// Phone: both sidebars are drawers.
    Compact,
}

impl LayoutMode {
    pub fn from_width(width: f32) -> Self {
        if width >= 1100.0 {
            Self::Wide
        } else if width >= 760.0 {
            Self::Medium
        } else {
            Self::Compact
        }
    }

    /// Whether the files sidebar is a drawer.
    pub fn left_drawer(self) -> bool {
        self == Self::Compact
    }

    /// Whether the outline/backlinks sidebar is a drawer.
    pub fn right_drawer(self) -> bool {
        self != Self::Wide
    }
}

fn update_layout_mode(window: Query<&Window, With<PrimaryWindow>>, nav: Res<Nav>) {
    let Ok(window) = window.single() else {
        return;
    };
    let mode = LayoutMode::from_width(window.width());
    if nav.mode.set_if_neq(mode) {
        nav.left_open.set_if_neq(false);
        nav.right_open.set_if_neq(false);
    }
}

/// Remembers whether the user is on a touch screen (last input was a touch).
fn detect_touch(
    mut touches: MessageReader<TouchInput>,
    mut mouse: MessageReader<MouseButtonInput>,
    nav: Res<Nav>,
) {
    if touches.read().last().is_some() {
        nav.touch.set_if_neq(true);
    } else if mouse.read().last().is_some() {
        nav.touch.set_if_neq(false);
    }
}

// ---------------------------------------------------------------------------
// Touch scrolling
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub(crate) struct TouchScroll {
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
    /// When a touch last stopped a fling (that touch must not also tap a link).
    caught: Option<f64>,
}

impl TouchScroll {
    /// Whether the touch that just ended only stopped a fling, so its tap should be ignored.
    pub(crate) fn take_caught(&mut self, now: f64) -> bool {
        self.caught.take().is_some_and(|t| now - t < 1.0)
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

    fn velocity(&self, now: f64) -> Vec2 {
        let Some(&(newest, _)) = self.samples.back() else {
            return Vec2::ZERO;
        };
        if now - newest > Self::STOPPED {
            return Vec2::ZERO;
        }
        // Samples within the horizon, without a pause between them.
        let mut used: Vec<(f64, Vec2)> = Vec::new();
        let mut previous = newest;
        for &(time, travel) in self.samples.iter().rev() {
            if newest - time > Self::HORIZON || previous - time > Self::STOPPED {
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

fn touch_drag_start(drag: On<PointerDragStart>, mut scroll: ResMut<TouchScroll>) {
    if !drag.pointer.id.is_touch() || drag.entity != drag.original_event_target() {
        return;
    }
    scroll.pending = Some((drag.entity, drag.pointer.id));
    scroll.active = None;
    scroll.coasting = None;
    scroll.fling = None;
    scroll.tracker.reset();
}

/// A finger landing on a fling stops it, like on Android; that touch must not also tap.
fn catch_fling(
    press: On<bevy::picking::events::PointerPress>,
    time: Res<Time<Real>>,
    mut scroll: ResMut<TouchScroll>,
) {
    if !press.pointer.id.is_touch() || press.entity != press.original_event_target() {
        return;
    }
    if scroll.coasting.take().is_some() {
        scroll.fling = None;
        scroll.caught = Some(time.elapsed_secs_f64());
    } else {
        scroll.caught = None;
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
    if let Some((entity, pointer)) = scroll.pending
        && pointer == drag.pointer.id
    {
        scroll.pending = None;
        scroll.active =
            scroll_target(entity, drag.distance, &parents, &areas).map(|e| (e, pointer));
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
    nav: Res<Nav>,
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
        let velocity = (scroll.tracker.velocity(now) * nav.scroll_factor())
            .clamp_length_max(MAX_FLING_VELOCITY);
        let start = areas.get(target).map_or(Vec2::ZERO, |(_, _, p)| p.0);
        scroll.fling = Fling::new(now, start, velocity);
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
    let (Some(target), Some(fling)) = (scroll.coasting, scroll.fling) else {
        return;
    };
    let (position, done) = fling.at(time.elapsed_secs_f64());
    let Ok((_, _, current)) = areas.get(target) else {
        scroll.coasting = None;
        return;
    };
    let moved = apply_scroll(target, position - current.0, &mut areas);
    // Stop at the end of the curve, or at an edge.
    if done || !moved {
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
    fn animating(&self) -> bool {
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
    nav: Res<Nav>,
    pixels_per_line: Res<bevy::input::mouse::MouseScrollPixelsPerLine>,
    mut areas: Query<(&Node, &mut SmoothScroll), With<bevy::ui_widgets::ScrollArea>>,
) {
    let Ok((node, mut smooth)) = areas.get_mut(scroll.entity) else {
        return;
    };
    let delta = scroll.to_pixels(&pixels_per_line);
    let mut extra = -Vec2::new(delta.x, delta.y) * (nav.scroll_factor() - 1.0);
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
        let touching =
            touch.active.is_some_and(|(e, _)| e == entity) || touch.coasting == Some(entity);
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

// ---------------------------------------------------------------------------
// Selection action bar
// ---------------------------------------------------------------------------

fn sync_selection_bar(selection: Res<TextSelection>, nav: Res<Nav>) {
    if selection.is_changed() {
        nav.has_selection.set_if_neq(!selection.is_collapsed());
    }
}

fn bar_button(label: &str) -> impl Scene + use<> {
    let text = Text::new(label);
    bsn! {
        Node {
            padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        bevy::ui_widgets::Button
        BackgroundColor(Color::NONE)
        Children [
            text @{crate::ui::ui_font(15.0, FontWeight::MEDIUM)} TextColor(palette::TEXT) Pickable::IGNORE
        ]
    }
}

/// Floating Copy / Select all / Done bar shown while text is selected on touch screens.
pub fn selection_bar(nav: Nav) -> impl Scene {
    let (has_selection, touch) = (nav.has_selection.clone(), nav.touch.clone());
    coat! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            bottom: Val::Px(24.0),
            justify_content: JustifyContent::Center,
        }
        GlobalZIndex(90)
        Pickable::IGNORE
        bevy_ui_textselect::PreserveSelection
        Children [
            if $has_selection && $touch {
                Node {
                    padding: UiRect::all(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    column_gap: Val::Px(2.0),
                }
                BackgroundColor(palette::BG_SIDEBAR)
                BorderColor::all(palette::BORDER)
                BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.5), Val::Px(0.0), Val::Px(6.0), Val::Px(0.0), Val::Px(20.0))
                Children [
                    @{bar_button("Copy")}
                    on(|_: On<Activate>, mut commands: Commands| {
                        commands.trigger(CopySelection);
                        commands.trigger(ClearSelection);
                    })
                    --
                    @{bar_button("Select all")}
                    on(|_: On<Activate>, mut commands: Commands| commands.trigger(SelectAll::default()))
                    --
                    @{bar_button("Done")}
                    on(|_: On<Activate>, mut commands: Commands| commands.trigger(ClearSelection))
                ]
            }
        ]
    }
}

// ---------------------------------------------------------------------------
// In-app keyboard
// ---------------------------------------------------------------------------

type Keys = VirtualKeyboard<&'static str>;

/// The text field the on-screen keyboard types into.
#[derive(Resource, Default)]
struct KeyboardTarget(Option<Entity>);

/// Root of the on-screen keyboard.
#[derive(Component, Clone, Copy, Debug, Default)]
struct KeyboardPanel;

fn track_keyboard_target(
    focus: Res<InputFocus>,
    editable: Query<(), With<EditableText>>,
    panel: Query<(), With<KeyboardPanel>>,
    parents: Query<&ChildOf>,
    mut target: ResMut<KeyboardTarget>,
    nav: Res<Nav>,
) {
    if !focus.is_changed() {
        return;
    }
    match focus.get() {
        Some(entity) if editable.contains(entity) => target.0 = Some(entity),
        // Pressing a key focuses the key's button: keep typing into the field.
        Some(entity) if parents.iter_ancestors(entity).any(|e| panel.contains(e)) => {}
        _ => target.0 = None,
    }
    nav.keyboard.set_if_neq(target.0.is_some());
}

const ROWS: [&[&str]; 4] = [
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    &["a", "s", "d", "f", "g", "h", "j", "k", "l"],
    &["z", "x", "c", "v", "b", "n", "m", "del"],
    &["space", "Done"],
];

fn on_virtual_key(
    key: On<VirtualKeyPressed<&'static str>>,
    mut target: ResMut<KeyboardTarget>,
    mut fields: Query<&mut EditableText>,
    mut focus: ResMut<InputFocus>,
    nav: Res<Nav>,
) {
    let Some(entity) = target.0 else {
        return;
    };
    match key.key {
        "Done" => {
            target.0 = None;
            focus.clear();
            nav.keyboard.set(false);
            return;
        }
        "del" => {
            if let Ok(mut field) = fields.get_mut(entity) {
                field.queue_edit(TextEdit::Backspace);
            }
        }
        k => {
            if let Ok(mut field) = fields.get_mut(entity) {
                let text = if k == "space" { " " } else { k };
                field.queue_edit(TextEdit::Insert(text.into()));
            }
        }
    }
    focus.set(entity, bevy::input_focus::FocusCause::Navigated);
}

/// The on-screen keyboard, docked at the bottom while a text field is focused by touch.
pub fn keyboard_panel(nav: Nav) -> impl Scene {
    let (keyboard, touch) = (nav.keyboard.clone(), nav.touch.clone());
    coat! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::End,
            flex_direction: FlexDirection::Column,
        }
        GlobalZIndex(120)
        Pickable::IGNORE
        Children [
            if $keyboard && $touch {
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(8.0), Val::Px(14.0)),
                    border: UiRect::top(Val::Px(1.0)),
                }
                BackgroundColor(palette::BG_SIDEBAR)
                BorderColor::all(palette::BORDER)
                KeyboardPanel
                Children [
                    @Keys {
                        @keys: {ROWS.iter().map(|row| row.to_vec()).collect::<Vec<_>>()}
                    }
                    Node { width: Val::Percent(100.0) }
                    on(on_virtual_key)
                ]
            }
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_of_a_steady_finger() {
        let mut tracker = VelocityTracker::default();
        // 1200 px/s downwards, sampled at 120 Hz.
        for i in 0..12 {
            tracker.add(f64::from(i) / 120.0, Vec2::new(0.0, 10.0));
        }
        let v = tracker.velocity(11.0 / 120.0);
        assert!((v.y - 1200.0).abs() < 1.0, "{v}");
        assert!(v.x.abs() < 1e-3);
        // A finger that rested for more than 40 ms before lifting does not fling.
        assert_eq!(tracker.velocity(11.0 / 120.0 + 0.05), Vec2::ZERO);
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
