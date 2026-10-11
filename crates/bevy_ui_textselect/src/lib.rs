//! Browser-like text selection for rich Bevy UI text.
//!
//! Bevy's [`Text`] + [`TextSpan`] hierarchy renders styled text, but it cannot be selected.
//! This crate adds selection on top of the [parley] layout Bevy already computes
//! ([`ComputedTextBlock::buffer`]), so hit-testing and highlight geometry are exact
//! for any font, ligature or bidi text:
//!
//! - Mark a `Text` root with [`SelectableText`].
//! - Optionally mark an ancestor container with [`SelectionGroup`]: a drag can then select
//!   across every selectable block inside it, in document (children) order, like a web page.
//! - Drag to select, double-click selects a word, triple-click a block, Shift+click extends.
//! - Ctrl/Cmd+C copies, Ctrl/Cmd+A selects the whole group, Esc clears,
//!   Shift+arrows/Home/End extend the selection.
//! - A click that is not a drag triggers [`TextSpanClick`] on the span under the pointer,
//!   bubbling up the hierarchy, which makes inline links trivial.
//! - Every selectable block gets an [`AccessibilityNode`] whose label mirrors its text.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_ui_textselect::prelude::*;
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Camera2d);
//!     commands.spawn((Node::default(), SelectionGroup)).with_children(|page| {
//!         page.spawn((Text::new("Select "), SelectableText))
//!             .with_child(TextSpan::new("me"));
//!     });
//! }
//! ```

use bevy::{
    a11y::AccessibilityNode,
    clipboard::Clipboard,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::InputFocus,
    picking::{
        cursor::EntityCursor,
        events::{PointerClick, PointerDrag, PointerPress, PointerRelease},
        hover::HoverMap,
        pointer::{PointerButton, PointerId},
    },
    prelude::*,
    text::{ComputedTextBlock, EditableText},
    ui::{ComputedUiRenderTargetInfo, UiGlobalTransform, UiScale, UiSystems},
    window::SystemCursorIcon,
};
use parley::{Affinity, Cursor, Selection};

pub mod text;

pub use text::{block_text, slice_text};

/// Common imports.
pub mod prelude {
    pub use crate::{
        ClearSelection, ClickableSpan, CopySelection, CopyText, Modifiers, SelectAll,
        SelectableText, SelectionGroup, SelectionStyle, TextPoint, TextSelectPlugin, TextSelection,
        TextSpanClick,
    };
}

/// Adds text selection support.
#[derive(Default)]
pub struct TextSelectPlugin;

impl Plugin for TextSelectPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<Clipboard>() {
            app.init_resource::<Clipboard>();
        }
        app.init_resource::<TextSelection>()
            .init_resource::<PointerQueue>()
            .add_observer(queue_press)
            .add_observer(queue_drag)
            .add_observer(queue_release)
            .add_observer(queue_click)
            .add_observer(on_copy)
            .add_observer(on_select_all)
            .add_observer(on_clear)
            .init_resource::<SelectionStyle>()
            .init_resource::<InputFocus>()
            .add_systems(
                PreUpdate,
                (pointer_selection, long_press_selection, keyboard_selection)
                    .chain()
                    .in_set(TextSelectSystems::Input)
                    .after(bevy::picking::PickingSystems::Last),
            )
            .add_systems(
                PreUpdate,
                hover_cursor
                    .after(bevy::picking::PickingSystems::Hover)
                    .before(bevy::picking::PickingSystems::Last),
            )
            .add_systems(
                PostUpdate,
                (
                    validate_selection,
                    update_accessibility,
                    draw_highlights.after(UiSystems::Layout),
                )
                    .chain()
                    .in_set(TextSelectSystems::Render)
                    .after(UiSystems::Layout),
            );
    }
}

/// System sets of this crate.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextSelectSystems {
    /// Pointer and keyboard handling, in `PreUpdate` after picking.
    Input,
    /// Highlight geometry and a11y labels, in `PostUpdate` after UI layout.
    Render,
}

/// Marks a [`Text`] root as selectable.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component, Default, Clone)]
#[require(Pickable, EntityCursor::System(SystemCursorIcon::Text))]
pub struct SelectableText;

/// A container whose selectable descendants form one continuous selection.
///
/// It also hosts the highlight layer, so highlights scroll and clip with the content.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component, Default, Clone)]
pub struct SelectionGroup;

/// Text placed on the clipboard instead of a span's visible text when the whole span is
/// selected (e.g. a rendered link copying its markdown source).
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default, Clone)]
pub struct CopyText(pub String);

/// Marks a span (or block) as clickable: shows a pointer cursor while hovered.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component, Default, Clone)]
pub struct ClickableSpan;

/// Visual style of selection highlights.
#[derive(Resource, Clone, Debug)]
pub struct SelectionStyle {
    /// Highlight color, drawn over the text (use transparency).
    pub color: Color,
}

impl Default for SelectionStyle {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.48, 0.42, 0.95, 0.35),
        }
    }
}

/// A position inside a selectable block: a byte index into its concatenated text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextPoint {
    /// The [`SelectableText`] entity.
    pub block: Entity,
    /// Byte index into [`block_text`].
    pub index: usize,
}

/// The current (single, global) text selection.
#[derive(Resource, Clone, Debug, Default)]
pub struct TextSelection {
    /// The [`SelectionGroup`] containing the selection (or the lone block).
    pub group: Option<Entity>,
    /// Where the selection started.
    pub anchor: Option<TextPoint>,
    /// Where the selection ends (moves while dragging).
    pub focus: Option<TextPoint>,
    dragging: Option<PointerId>,
    long_press: Option<LongPress>,
}

/// A touch press that becomes a word selection if held still.
#[derive(Clone, Copy, Debug)]
struct LongPress {
    pointer: PointerId,
    block: Entity,
    position: Vec2,
    elapsed: f32,
}

/// How long a touch must be held still to select a word, in seconds.
pub const LONG_PRESS_SECONDS: f32 = 0.45;
/// How far a touch may move (logical px) before it counts as a scroll instead of a press.
pub const TOUCH_SLOP: f32 = 10.0;

impl TextSelection {
    /// Removes the selection.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// `true` while a touch that started a selection (by long-press) is still down; touch
    /// scrolling should be suspended so the finger can extend the selection.
    pub fn is_touch_selecting(&self) -> bool {
        self.dragging.is_some_and(|p| p.is_touch())
    }

    /// `true` if nothing is selected.
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    /// Selects `[start, end)` of a single block.
    pub fn select_block_range(&mut self, group: Entity, block: Entity, start: usize, end: usize) {
        self.group = Some(group);
        self.anchor = Some(TextPoint {
            block,
            index: start,
        });
        self.focus = Some(TextPoint { block, index: end });
    }

    /// Selected byte ranges per block, given the group's blocks in document order.
    pub fn ranges(
        &self,
        blocks: &[Entity],
        len: impl Fn(Entity) -> usize,
    ) -> Vec<(Entity, usize, usize)> {
        let (Some(a), Some(f)) = (self.anchor, self.focus) else {
            return Vec::new();
        };
        let (Some(ai), Some(fi)) = (
            blocks.iter().position(|b| *b == a.block),
            blocks.iter().position(|b| *b == f.block),
        ) else {
            return Vec::new();
        };
        let (start, end, si, ei) = if (ai, a.index) <= (fi, f.index) {
            (a, f, ai, fi)
        } else {
            (f, a, fi, ai)
        };
        let mut out = Vec::new();
        for (i, block) in blocks.iter().enumerate().take(ei + 1).skip(si) {
            let from = if i == si { start.index } else { 0 };
            let to = if i == ei { end.index } else { len(*block) };
            if from < to {
                out.push((*block, from, to));
            }
        }
        out
    }
}

/// Triggered when a selectable text is clicked without dragging. Targets the span under
/// the pointer (or the root `Text`), and bubbles up through [`ChildOf`].
#[derive(EntityEvent, Clone, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct TextSpanClick {
    /// The span entity (propagates to its ancestors).
    pub entity: Entity,
    /// The [`SelectableText`] block that was clicked.
    pub block: Entity,
    /// Byte index of the click inside the block.
    pub index: usize,
    /// The clicked span (stays the same while bubbling).
    pub span: Entity,
}

/// Marker for the highlight layer spawned inside a group.
#[derive(Component)]
struct HighlightLayer;

/// Marker for one highlight rectangle.
#[derive(Component)]
struct HighlightRect;

#[derive(SystemParam)]
struct Blocks<'w, 's> {
    texts: Query<
        'w,
        's,
        (
            &'static ComputedTextBlock,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static ComputedUiRenderTargetInfo,
        ),
        With<SelectableText>,
    >,
    selectable: Query<'w, 's, (), With<SelectableText>>,
    groups: Query<'w, 's, (), With<SelectionGroup>>,
    parents: Query<'w, 's, &'static ChildOf>,
    children: Query<'w, 's, &'static Children>,
    text: Query<'w, 's, &'static Text>,
    spans: Query<'w, 's, &'static TextSpan>,
    copy: Query<'w, 's, &'static CopyText>,
    ui_scale: Res<'w, UiScale>,
}

use bevy::ecs::system::SystemParam;

impl Blocks<'_, '_> {
    /// The selectable block of a picked entity: the block itself, or the `Text` root of a
    /// picked [`TextSpan`] (Bevy picks individual spans).
    fn block_of(&self, entity: Entity) -> Option<Entity> {
        if self.selectable.contains(entity) {
            return Some(entity);
        }
        if !self.spans.contains(entity) {
            return None;
        }
        self.parents
            .iter_ancestors(entity)
            .find(|e| self.selectable.contains(*e))
    }

    /// The group a block belongs to (the block itself if it has no group ancestor).
    fn group_of(&self, block: Entity) -> Entity {
        let mut current = block;
        loop {
            if self.groups.contains(current) {
                return current;
            }
            match self.parents.get(current) {
                Ok(parent) => current = parent.parent(),
                Err(_) => return block,
            }
        }
    }

    /// Selectable blocks of a group in depth-first children order.
    fn blocks_of(&self, group: Entity) -> Vec<Entity> {
        let mut out = Vec::new();
        let mut stack = vec![group];
        while let Some(e) = stack.pop() {
            if self.selectable.contains(e) {
                out.push(e);
                // Text spans are never blocks.
                continue;
            }
            if let Ok(children) = self.children.get(e) {
                stack.extend(children.iter().rev());
            }
        }
        out
    }

    fn text_of(&self, block: Entity) -> String {
        self.texts
            .get(block)
            .map(|(computed, ..)| block_text(computed, &self.text, &self.spans))
            .unwrap_or_default()
    }

    /// Pointer position (window logical px) → point in the block's layout space (physical px).
    fn layout_point(&self, block: Entity, pointer: Vec2, clamp: bool) -> Option<Vec2> {
        let (_, node, transform, target) = self.texts.get(block).ok()?;
        let inverse = transform.try_inverse()?;
        let local = inverse.transform_point2(pointer * target.scale_factor() / self.ui_scale.0);
        let content = node.content_box();
        let local = if clamp {
            content.clamp_point(local)
        } else {
            local
        };
        Some(local - content.min)
    }

    /// Bounding rect of a block in UI physical space.
    fn global_rect(&self, block: Entity) -> Option<(Rect, f32, f32)> {
        let (_, node, transform, target) = self.texts.get(block).ok()?;
        let center = transform.transform_point2(Vec2::ZERO);
        Some((
            Rect::from_center_size(center, node.size),
            target.scale_factor(),
            self.ui_scale.0,
        ))
    }

    /// Hit-tests a point; returns the byte index.
    fn index_at(&self, block: Entity, pointer: Vec2) -> Option<usize> {
        let point = self.layout_point(block, pointer, true)?;
        let (computed, ..) = self.texts.get(block).ok()?;
        Some(Cursor::from_point(computed.buffer(), point.x, point.y).index())
    }

    /// The block of `group` under or vertically nearest to the pointer.
    fn nearest_block(&self, group: Entity, pointer: Vec2) -> Option<Entity> {
        let blocks = self.blocks_of(group);
        let mut best: Option<(f32, Entity)> = None;
        for block in blocks {
            let Some((rect, scale, ui_scale)) = self.global_rect(block) else {
                continue;
            };
            let p = pointer * scale / ui_scale;
            let dy = if p.y < rect.min.y {
                rect.min.y - p.y
            } else if p.y > rect.max.y {
                p.y - rect.max.y
            } else {
                0.0
            };
            let dx = if p.x < rect.min.x {
                rect.min.x - p.x
            } else if p.x > rect.max.x {
                p.x - rect.max.x
            } else {
                0.0
            };
            // Prefer vertical proximity; horizontal distance breaks ties (table cells).
            let score = dy * 1000.0 + dx;
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, block));
            }
        }
        best.map(|(_, b)| b)
    }

    /// The span entity containing `index`.
    fn span_at(&self, block: Entity, index: usize) -> Option<Entity> {
        let (computed, ..) = self.texts.get(block).ok()?;
        text::span_at(computed, &self.text, &self.spans, index)
    }

    fn copy_text(&self, selection: &TextSelection) -> String {
        let Some(group) = selection.group else {
            return String::new();
        };
        let blocks = self.blocks_of(group);
        let ranges = selection.ranges(&blocks, |b| self.text_of(b).len());
        let mut out = String::new();
        for (i, (block, start, end)) in ranges.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if let Ok((computed, ..)) = self.texts.get(*block) {
                out.push_str(&slice_text(
                    computed,
                    &self.text,
                    &self.spans,
                    &self.copy,
                    *start..*end,
                ));
            }
        }
        out
    }
}

/// Modifier keys, replayed in event order.
///
/// `ButtonInput` only reflects the state at the end of the frame, so a fast `Ctrl+C` whose
/// press and release land in the same frame would be missed. This starts from the state
/// before the frame's events and is updated by each [`KeyboardInput`] in order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Modifiers {
    /// Either Control key.
    pub ctrl: bool,
    /// Either Shift key.
    pub shift: bool,
    /// Either Alt/Option key.
    pub alt: bool,
    /// Either Super/Cmd key.
    pub super_key: bool,
}

const MODIFIER_KEYS: [KeyCode; 8] = [
    KeyCode::ControlLeft,
    KeyCode::ControlRight,
    KeyCode::ShiftLeft,
    KeyCode::ShiftRight,
    KeyCode::AltLeft,
    KeyCode::AltRight,
    KeyCode::SuperLeft,
    KeyCode::SuperRight,
];

impl Modifiers {
    /// Modifier state before this frame's keyboard events.
    pub fn before_frame(keys: &ButtonInput<KeyCode>) -> Self {
        let mut m = Self::default();
        for key in MODIFIER_KEYS {
            let held = (keys.pressed(key) && !keys.just_pressed(key))
                || (keys.just_released(key) && !keys.just_pressed(key));
            if held {
                m.set(key, true);
            }
        }
        m
    }

    /// Applies one keyboard event.
    pub fn update(&mut self, input: &KeyboardInput) {
        if MODIFIER_KEYS.contains(&input.key_code) {
            self.set(input.key_code, input.state == ButtonState::Pressed);
        }
    }

    fn set(&mut self, key: KeyCode, held: bool) {
        let flag = match key {
            KeyCode::ControlLeft | KeyCode::ControlRight => &mut self.ctrl,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => &mut self.shift,
            KeyCode::AltLeft | KeyCode::AltRight => &mut self.alt,
            KeyCode::SuperLeft | KeyCode::SuperRight => &mut self.super_key,
            _ => return,
        };
        *flag = held;
    }

    /// The platform's command modifier: Cmd on macOS hosts, Ctrl elsewhere.
    pub fn command(&self) -> bool {
        if cfg!(target_os = "macos") || mac_host() {
            self.super_key
        } else {
            self.ctrl
        }
    }
}

fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

#[allow(clippy::too_many_arguments)]
/// Pointer events captured by observers (once, at their original target), processed in order
/// by [`pointer_selection`].
#[derive(Resource, Default)]
struct PointerQueue {
    presses: Vec<PointerPress>,
    drags: Vec<PointerDrag>,
    releases: Vec<PointerRelease>,
    clicks: Vec<PointerClick>,
}

fn queue_press(event: On<PointerPress>, mut queue: ResMut<PointerQueue>) {
    if event.entity == event.original_event_target() {
        queue.presses.push(event.event().clone());
    }
}

fn queue_drag(event: On<PointerDrag>, mut queue: ResMut<PointerQueue>) {
    if event.entity == event.original_event_target() {
        queue.drags.push(event.event().clone());
    }
}

fn queue_release(event: On<PointerRelease>, mut queue: ResMut<PointerQueue>) {
    if event.entity == event.original_event_target() {
        queue.releases.push(event.event().clone());
    }
}

fn queue_click(event: On<PointerClick>, mut queue: ResMut<PointerQueue>) {
    if event.entity == event.original_event_target() {
        queue.clicks.push(event.event().clone());
    }
}

fn pointer_selection(
    mut queue: ResMut<PointerQueue>,
    mut selection: ResMut<TextSelection>,
    keys: Res<ButtonInput<KeyCode>>,
    blocks: Blocks,
    mut commands: Commands,
) {
    let PointerQueue {
        presses,
        drags,
        releases,
        clicks,
    } = std::mem::take(&mut *queue);
    for press in &presses {
        if press.button != PointerButton::Primary {
            continue;
        }
        if press.pointer.id.is_touch() {
            // Touch: a tap clears the selection and drags scroll; holding still selects.
            selection.long_press = blocks.block_of(press.entity).map(|block| LongPress {
                pointer: press.pointer.id,
                block,
                position: press.pointer.position,
                elapsed: 0.0,
            });
            // Tapping text clears a selection; tapping elsewhere (e.g. a "Copy" button) keeps it.
            if selection.long_press.is_some() && !selection.is_collapsed() {
                selection.clear();
                selection.long_press = None;
            }
            continue;
        }
        let Some(block) = blocks.block_of(press.entity) else {
            if selection.anchor.is_some() {
                selection.clear();
            }
            continue;
        };
        let group = blocks.group_of(block);
        let Some(index) = blocks.index_at(block, press.pointer.position) else {
            continue;
        };
        let Ok((computed, ..)) = blocks.texts.get(block) else {
            continue;
        };
        let layout = computed.buffer();
        match press.count {
            1 if shift_pressed(&keys) && selection.group == Some(group) => {
                selection.focus = Some(TextPoint { block, index });
            }
            1 => {
                selection.group = Some(group);
                selection.anchor = Some(TextPoint { block, index });
                selection.focus = selection.anchor;
            }
            2 => {
                let Some(p) = blocks.layout_point(block, press.pointer.position, true) else {
                    continue;
                };
                let word = Selection::word_from_point(layout, p.x, p.y).text_range();
                selection.select_block_range(group, block, word.start, word.end);
            }
            _ => {
                let len = blocks.text_of(block).len();
                selection.select_block_range(group, block, 0, len);
            }
        }
        selection.dragging = Some(press.pointer.id);
    }

    for drag in &drags {
        if selection
            .long_press
            .is_some_and(|lp| lp.pointer == drag.pointer.id && drag.distance.length() > TOUCH_SLOP)
        {
            selection.long_press = None;
        }
        if drag.button != PointerButton::Primary || selection.dragging != Some(drag.pointer.id) {
            continue;
        }
        let Some(group) = selection.group else {
            continue;
        };
        let Some(block) = blocks.nearest_block(group, drag.pointer.position) else {
            continue;
        };
        if let Some(index) = blocks.index_at(block, drag.pointer.position) {
            selection.focus = Some(TextPoint { block, index });
        }
    }

    for release in &releases {
        if release.button == PointerButton::Primary {
            selection.dragging = None;
            selection.long_press = None;
        }
    }

    for click in &clicks {
        if click.button != PointerButton::Primary {
            continue;
        }
        let Some(block) = blocks.block_of(click.entity) else {
            continue;
        };
        // A drag that selected something is not a click.
        if !selection.is_collapsed() {
            continue;
        }
        let Some(index) = blocks.index_at(block, click.pointer.position) else {
            continue;
        };
        // Bevy picks individual spans; a click on the root text (e.g. past the end of a
        // line) only counts when it lands on a glyph line.
        let span = if blocks.spans.contains(click.entity) {
            click.entity
        } else {
            let inside = blocks
                .layout_point(block, click.pointer.position, false)
                .zip(blocks.texts.get(block).ok())
                .is_some_and(|(p, (c, ..))| text::point_inside_text(c.buffer(), p));
            if !inside {
                continue;
            }
            blocks.span_at(block, index).unwrap_or(block)
        };
        commands.trigger(TextSpanClick {
            entity: span,
            block,
            index,
            span,
        });
    }
}

/// Turns a held, still touch into a word selection that the finger can then extend.
fn long_press_selection(
    time: Res<Time<Real>>,
    mut selection: ResMut<TextSelection>,
    blocks: Blocks,
) {
    let Some(mut lp) = selection.long_press else {
        return;
    };
    lp.elapsed += time.delta_secs();
    if lp.elapsed < LONG_PRESS_SECONDS {
        selection.long_press = Some(lp);
        return;
    }
    selection.long_press = None;
    let Some(point) = blocks.layout_point(lp.block, lp.position, true) else {
        return;
    };
    let Ok((computed, ..)) = blocks.texts.get(lp.block) else {
        return;
    };
    let word = Selection::word_from_point(computed.buffer(), point.x, point.y).text_range();
    let group = blocks.group_of(lp.block);
    selection.select_block_range(group, lp.block, word.start, word.end);
    selection.dragging = Some(lp.pointer);
}

/// Copies the current selection to the clipboard (e.g. from a "Copy" button).
#[derive(Event, Clone, Copy, Debug, Default)]
pub struct CopySelection;

/// Selects every block of the current selection's group, or of `group` if given.
#[derive(Event, Clone, Copy, Debug, Default)]
pub struct SelectAll {
    /// The [`SelectionGroup`] to select; defaults to the current selection's group.
    pub group: Option<Entity>,
}

/// Clears the selection.
#[derive(Event, Clone, Copy, Debug, Default)]
pub struct ClearSelection;

fn on_copy(
    _: On<CopySelection>,
    selection: Res<TextSelection>,
    mut clipboard: ResMut<Clipboard>,
    blocks: Blocks,
) {
    if !selection.is_collapsed()
        && let Err(err) = clipboard.set_text(blocks.copy_text(&selection))
    {
        warn!("could not copy selection: {err:?}");
    }
}

fn on_select_all(event: On<SelectAll>, mut selection: ResMut<TextSelection>, blocks: Blocks) {
    let Some(group) = event.group.or(selection.group) else {
        return;
    };
    let all = blocks.blocks_of(group);
    if let (Some(first), Some(last)) = (all.first(), all.last()) {
        selection.group = Some(group);
        selection.anchor = Some(TextPoint {
            block: *first,
            index: 0,
        });
        selection.focus = Some(TextPoint {
            block: *last,
            index: blocks.text_of(*last).len(),
        });
    }
}

fn on_clear(_: On<ClearSelection>, mut selection: ResMut<TextSelection>) {
    selection.clear();
}

fn keyboard_selection(
    mut keys_in: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    editable: Query<(), With<EditableText>>,
    mut selection: ResMut<TextSelection>,
    mut clipboard: ResMut<Clipboard>,
    blocks: Blocks,
) {
    // A focused text input owns the keyboard.
    if focus.get().is_some_and(|e| editable.contains(e)) {
        keys_in.clear();
        return;
    }
    let mut modifiers = Modifiers::before_frame(&keys);
    for input in keys_in.read() {
        modifiers.update(input);
        if input.state != ButtonState::Pressed {
            continue;
        }
        let (command, shift) = (modifiers.command(), modifiers.shift);
        match &input.logical_key {
            Key::Character(c) if command && c.eq_ignore_ascii_case("c") => {
                if !selection.is_collapsed() {
                    let text = blocks.copy_text(&selection);
                    if let Err(err) = clipboard.set_text(text) {
                        warn!("could not copy selection: {err:?}");
                    }
                }
            }
            Key::Copy => {
                let text = blocks.copy_text(&selection);
                let _ = clipboard.set_text(text);
            }
            Key::Character(c) if command && c.eq_ignore_ascii_case("a") => {
                let Some(group) = selection.group else {
                    continue;
                };
                let all = blocks.blocks_of(group);
                if let (Some(first), Some(last)) = (all.first(), all.last()) {
                    selection.anchor = Some(TextPoint {
                        block: *first,
                        index: 0,
                    });
                    selection.focus = Some(TextPoint {
                        block: *last,
                        index: blocks.text_of(*last).len(),
                    });
                }
            }
            Key::Escape => selection.clear(),
            key @ (Key::ArrowLeft
            | Key::ArrowRight
            | Key::ArrowUp
            | Key::ArrowDown
            | Key::Home
            | Key::End)
                if shift =>
            {
                extend_with_key(key, command, &mut selection, &blocks);
            }
            _ => {}
        }
    }
}

fn extend_with_key(key: &Key, word: bool, selection: &mut TextSelection, blocks: &Blocks) {
    let (Some(group), Some(focus)) = (selection.group, selection.focus) else {
        return;
    };
    let Ok((computed, ..)) = blocks.texts.get(focus.block) else {
        return;
    };
    let layout = computed.buffer();
    let len = blocks.text_of(focus.block).len();
    let current = Selection::from_byte_index(layout, focus.index, Affinity::Downstream);
    let moved = match key {
        Key::ArrowLeft if word => current.previous_visual_word(layout, true),
        Key::ArrowRight if word => current.next_visual_word(layout, true),
        Key::ArrowLeft => current.previous_visual(layout, true),
        Key::ArrowRight => current.next_visual(layout, true),
        Key::ArrowUp => current.previous_line(layout, true),
        Key::ArrowDown => current.next_line(layout, true),
        Key::Home => current.line_start(layout, true),
        _ => current.line_end(layout, true),
    };
    let index = moved.focus().index();
    // Cross into neighbouring blocks at the edges.
    let all = blocks.blocks_of(group);
    let pos = all.iter().position(|b| *b == focus.block);
    let at_start = index == focus.index && focus.index == 0;
    let at_end = index == focus.index && focus.index >= len;
    match (pos, key) {
        (Some(p), Key::ArrowLeft | Key::ArrowUp) if at_start && p > 0 => {
            let prev = all[p - 1];
            selection.focus = Some(TextPoint {
                block: prev,
                index: blocks.text_of(prev).len(),
            });
        }
        (Some(p), Key::ArrowRight | Key::ArrowDown) if at_end && p + 1 < all.len() => {
            selection.focus = Some(TextPoint {
                block: all[p + 1],
                index: 0,
            });
        }
        _ => {
            selection.focus = Some(TextPoint {
                block: focus.block,
                index,
            });
        }
    }
}

/// Whether shortcuts should use Cmd (macOS hosts, including wasm running on a Mac).
fn mac_host() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use std::sync::OnceLock;
        static MAC: OnceLock<bool> = OnceLock::new();
        *MAC.get_or_init(text::web_is_mac)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Drops the selection when its blocks despawn (e.g. on navigation).
fn validate_selection(mut selection: ResMut<TextSelection>, blocks: Blocks) {
    let alive = |p: Option<TextPoint>| p.is_none_or(|p| blocks.texts.contains(p.block));
    if selection.anchor.is_some() && !(alive(selection.anchor) && alive(selection.focus)) {
        selection.clear();
    }
}

fn update_accessibility(
    mut commands: Commands,
    changed: Query<
        (Entity, &ComputedTextBlock, Option<&mut AccessibilityNode>),
        (With<SelectableText>, Changed<ComputedTextBlock>),
    >,
    text: Query<&Text>,
    spans: Query<&TextSpan>,
) {
    for (entity, computed, node) in changed {
        let label = block_text(computed, &text, &spans);
        if let Some(mut node) = node {
            if node.label().is_none_or(|l| *l != *label) {
                node.set_label(label);
            }
        } else {
            let mut node = accesskit::Node::new(accesskit::Role::Paragraph);
            node.set_label(label);
            commands.entity(entity).insert(AccessibilityNode(node));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_highlights(
    mut commands: Commands,
    selection: Res<TextSelection>,
    style: Res<SelectionStyle>,
    blocks: Blocks,
    groups: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<SelectionGroup>>,
    layers: Query<(Entity, &ChildOf, Option<&Children>), With<HighlightLayer>>,
    mut last: Local<
        Option<(
            Option<Entity>,
            Option<TextPoint>,
            Option<TextPoint>,
            Vec<Vec2>,
        )>,
    >,
) {
    // Collect rects for the active selection in the group's padding-box space (logical px).
    let mut rects: Vec<Rect> = Vec::new();
    let mut transforms_key: Vec<Vec2> = Vec::new();
    let group = selection.group;
    if let Some(group) = group
        && !selection.is_collapsed()
        && let Ok((_, group_node, group_transform)) = groups.get(group)
        && let Some(group_inverse) = group_transform.try_inverse()
    {
        let all = blocks.blocks_of(group);
        let ranges = selection.ranges(&all, |b| blocks.text_of(b).len());
        let origin = group_node.padding_box().min;
        let inv = group_node.inverse_scale_factor;
        for (block, start, end) in ranges {
            let Ok((computed, node, transform, _)) = blocks.texts.get(block) else {
                continue;
            };
            transforms_key.push(transform.transform_point2(Vec2::ZERO));
            let layout = computed.buffer();
            let sel = Selection::new(
                Cursor::from_byte_index(layout, start, Affinity::Downstream),
                Cursor::from_byte_index(layout, end, Affinity::Upstream),
            );
            let content_min = node.content_box().min;
            for (bbox, _line) in sel.geometry(layout) {
                let to_group = |x: f64, y: f64| {
                    let ui =
                        transform.transform_point2(content_min + Vec2::new(x as f32, y as f32));
                    (group_inverse.transform_point2(ui) - origin) * inv
                };
                let min = to_group(bbox.x0, bbox.y0);
                let max = to_group(bbox.x1, bbox.y1);
                rects.push(Rect::from_corners(min, max));
            }
        }
        if let Ok((_, gn, gt)) = groups.get(group) {
            transforms_key.push(gt.transform_point2(Vec2::ZERO));
            transforms_key.push(gn.size);
        }
    }

    let key = (group, selection.anchor, selection.focus, transforms_key);
    if last.as_ref() == Some(&key) {
        return;
    }
    *last = Some(key);

    // Remove stale layers / rects.
    for (layer, parent, children) in &layers {
        if Some(parent.parent()) != group || rects.is_empty() {
            commands.entity(layer).despawn();
        } else if let Some(children) = children {
            for child in children {
                commands.entity(*child).despawn();
            }
        }
    }
    let (Some(group), false) = (group, rects.is_empty()) else {
        return;
    };
    let layer = layers
        .iter()
        .find(|(_, parent, _)| parent.parent() == group)
        .map_or_else(
            || {
                commands
                    .spawn((
                        HighlightLayer,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ZIndex(1000),
                        Pickable::IGNORE,
                        ChildOf(group),
                    ))
                    .id()
            },
            |(e, ..)| e,
        );
    for rect in rects {
        commands.spawn((
            HighlightRect,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(rect.min.x),
                top: Val::Px(rect.min.y),
                width: Val::Px(rect.width()),
                height: Val::Px(rect.height()),
                ..default()
            },
            BackgroundColor(style.color),
            Pickable::IGNORE,
            ChildOf(layer),
        ));
    }
}

/// Shows an I-beam over selectable text and a pointer over [`ClickableSpan`]s, by updating
/// the block's [`EntityCursor`] (applied by Bevy's cursor plugin).
fn hover_cursor(
    hover: Res<HoverMap>,
    blocks: Blocks,
    clickable: Query<(), With<ClickableSpan>>,
    mut cursors: Query<&mut EntityCursor, With<SelectableText>>,
    selection: Res<TextSelection>,
) {
    let Some(hits) = hover.get(&PointerId::Mouse) else {
        return;
    };

    for entity in hits.keys() {
        let Some(block) = blocks.block_of(*entity) else {
            continue;
        };
        let Ok(mut cursor) = cursors.get_mut(block) else {
            continue;
        };
        // Bevy picks individual spans, so the hit itself tells us whether it is a link.
        let over_link = selection.dragging.is_none() && clickable.contains(*entity);
        let icon = if over_link {
            SystemCursorIcon::Pointer
        } else {
            SystemCursorIcon::Text
        };
        let desired = EntityCursor::System(icon);
        if *cursor != desired {
            *cursor = desired;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_span_blocks_in_order() {
        let blocks: Vec<Entity> = (1..=3).map(|i| Entity::from_raw_u32(i).unwrap()).collect();
        let len = |_: Entity| 10;
        let mut selection = TextSelection {
            group: Some(blocks[0]),
            anchor: Some(TextPoint {
                block: blocks[2],
                index: 4,
            }),
            focus: Some(TextPoint {
                block: blocks[0],
                index: 7,
            }),
            dragging: None,
            long_press: None,
        };
        // Backwards selections are normalised.
        assert_eq!(
            selection.ranges(&blocks, len),
            vec![(blocks[0], 7, 10), (blocks[1], 0, 10), (blocks[2], 0, 4)]
        );
        selection.focus = selection.anchor;
        assert!(selection.is_collapsed());
        assert_eq!(selection.ranges(&blocks, len), Vec::new());
        selection.select_block_range(blocks[0], blocks[1], 2, 5);
        assert_eq!(selection.ranges(&blocks, len), vec![(blocks[1], 2, 5)]);
    }
}
