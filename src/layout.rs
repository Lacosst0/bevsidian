//! Responsive layout and touch support.
//!
//! - [`LayoutMode`] follows the window width: sidebars are inline on wide screens and become
//!   slide-over drawers on tablets and phones.
//! - Touch drags scroll the nearest [`ScrollArea`] with momentum (mouse drags select text).
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
    ui_widgets::{Activate, ScrollArea},
    window::PrimaryWindow,
};
use bevy_radioactive::coat::coat;
use bevy_ui_textselect::{ClearSelection, CopySelection, SelectAll, TextSelection};

use crate::{nav::Nav, ui::palette};

pub fn plugin(app: &mut App) {
    app.init_resource::<TouchScroll>()
        .init_resource::<KeyboardTarget>()
        .add_observer(touch_drag_start)
        .add_observer(touch_drag)
        .add_observer(touch_drag_end)
        .add_systems(
            Update,
            (
                update_layout_mode,
                detect_touch,
                touch_momentum,
                sync_selection_bar,
                track_keyboard_target,
            ),
        );
}

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
struct TouchScroll {
    /// Scroll area being dragged, and the pointer dragging it.
    active: Option<(Entity, bevy::picking::pointer::PointerId)>,
    /// Scroll area coasting after release.
    coasting: Option<Entity>,
    /// Velocity in logical px per second.
    velocity: Vec2,
}

/// The nearest scrollable ancestor (or self) of `entity`.
fn scroll_target(
    entity: Entity,
    parents: &Query<&ChildOf>,
    areas: &Query<(&Node, &ComputedNode), With<ScrollArea>>,
) -> Option<Entity> {
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find(|e| {
            areas.get(*e).is_ok_and(|(node, computed)| {
                let overflow = computed.content_size() - computed.size();
                (node.overflow.y == OverflowAxis::Scroll && overflow.y > 1.0)
                    || (node.overflow.x == OverflowAxis::Scroll && overflow.x > 1.0)
            })
        })
}

fn touch_drag_start(
    drag: On<PointerDragStart>,
    parents: Query<&ChildOf>,
    areas: Query<(&Node, &ComputedNode), With<ScrollArea>>,
    mut scroll: ResMut<TouchScroll>,
) {
    if !drag.pointer.id.is_touch() || drag.entity != drag.original_event_target() {
        return;
    }
    scroll.active = scroll_target(drag.entity, &parents, &areas).map(|e| (e, drag.pointer.id));
    scroll.coasting = None;
    scroll.velocity = Vec2::ZERO;
}

fn touch_drag(
    drag: On<PointerDrag>,
    time: Res<Time<Real>>,
    selection: Res<TextSelection>,
    mut scroll: ResMut<TouchScroll>,
    mut areas: Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
) {
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
    if apply_scroll(target, -drag.delta, &mut areas) {
        let dt = time.delta_secs().max(1.0 / 240.0);
        let instant = -drag.delta / dt;
        scroll.velocity = scroll.velocity.lerp(instant, 0.4);
    }
}

fn touch_drag_end(drag: On<PointerDragEnd>, mut scroll: ResMut<TouchScroll>) {
    if let Some((target, pointer)) = scroll.active
        && pointer == drag.pointer.id
    {
        scroll.active = None;
        scroll.coasting = Some(target);
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
    let Some(target) = scroll.coasting else {
        return;
    };
    let dt = time.delta_secs();
    let velocity = scroll.velocity;
    let moved = apply_scroll(target, velocity * dt, &mut areas);
    scroll.velocity *= (-4.0 * dt).exp();
    if !moved || scroll.velocity.length() < 20.0 {
        scroll.coasting = None;
        scroll.velocity = Vec2::ZERO;
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
    &["z", "x", "c", "v", "b", "n", "m", "←"],
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
        "←" => {
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
