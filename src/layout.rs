//! Responsive layout and touch support.
//!
//! - [`LayoutMode`] follows the window width: sidebars are inline on wide screens and become
//!   slide-over drawers on tablets and phones.
//! - Touch drags scroll and fling ([`bevy_ui_touchscroll`]), at the scroll speed setting.
//! - On touch devices, a selection action bar offers Copy / Select all, and focusing a text
//!   field shows an in-app keyboard (a Feathers [`VirtualKeyboard`]), since a WebGL canvas
//!   cannot summon the phone's keyboard.

use bevy::{
    feathers::controls::{VirtualKeyPressed, VirtualKeyboard},
    input::{
        mouse::MouseButtonInput,
        touch::{TouchInput, Touches},
    },
    input_focus::InputFocus,
    prelude::*,
    text::{EditableText, FontWeight, TextEdit},
    ui_widgets::Activate,
    window::PrimaryWindow,
};
use std::time::Duration;

use bevy_radioactive::coat::coat;
use bevy_ui_textselect::{ClearSelection, CopySelection, SelectAll, TextSelection};
use bevy_ui_touchscroll::{ScrollSpeed, SmoothScroll, TouchScroll, TouchScrollPlugin};

use bevsidian_core::{
    Nav, View,
    nav::LayoutMode,
    theme::{palette, ui_font},
};

pub fn plugin(app: &mut App) {
    app.add_plugins(TouchScrollPlugin)
        .init_resource::<KeyboardTarget>()
        .add_systems(
            Update,
            (
                update_layout_mode,
                detect_touch,
                sync_scroll_speed,
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
    graph: Res<bevsidian_graph::GraphState>,
    nav: Res<Nav>,
    focus: Res<InputFocus>,
    editable: Query<(), With<EditableText>>,
    layout_changed: Query<(), Or<(Changed<ComputedNode>, Changed<ScrollPosition>, Added<Node>)>>,
    smooth: Query<&SmoothScroll>,
    touches: Res<Touches>,
    time: Res<Time<Real>>,
    mut winit: ResMut<bevy::winit::WinitSettings>,
    mut awake_until: Local<f64>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    // A finger on the screen counts as busy: rendering at full rate from the touch on means the
    // first moves of a scroll are not slowed by the display ramping up from idle.
    let busy = scroll.busy()
        || touches.iter().next().is_some()
        || selection.is_busy()
        || (nav.view.get_untracked() == View::Graph && graph.animating())
        || nav.anchor.with_untracked(Option::is_some)
        || focus.get().is_some_and(|e| editable.contains(e))
        || smooth.iter().any(SmoothScroll::animating);
    let now = time.elapsed_secs_f64();
    if busy || !layout_changed.is_empty() {
        // Keep going a little longer: layout, text and reactive regions settle after a change,
        // and the next flick of a scroll often follows right away.
        *awake_until = now + AWAKE_TAIL;
    }
    let cap = nav.max_fps.get_untracked();
    let mode = if now < *awake_until {
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

/// How long (s) frames keep coming after the last change or interaction.
const AWAKE_TAIL: f64 = 0.5;

/// Update mode while nothing animates: wake on input, or once a second.
pub const IDLE: bevy::winit::UpdateMode = bevy::winit::UpdateMode::Reactive {
    wait: Duration::from_secs(1),
    react_to_device_events: true,
    react_to_user_events: true,
    react_to_window_events: true,
};

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

/// Applies the scroll speed setting to wheel and fling distances.
fn sync_scroll_speed(nav: Res<Nav>, mut speed: ResMut<ScrollSpeed>) {
    speed.set_if_neq(ScrollSpeed(nav.scroll_factor()));
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
            text @{ui_font(15.0, FontWeight::MEDIUM)} TextColor(palette::TEXT) Pickable::IGNORE
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

const ROWS: [&[&str]; 5] = [
    &["#", "*", "-", "`", "[", "]", "(", ")", ".", ","],
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    &["a", "s", "d", "f", "g", "h", "j", "k", "l"],
    &["z", "x", "c", "v", "b", "n", "m", "del"],
    &["space", "enter", "Done"],
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
                let text = match k {
                    "space" => " ",
                    // New line in the note editor; single-line fields ignore it.
                    "enter" if field.allow_newlines => "\n",
                    "enter" => "",
                    k => k,
                };
                if !text.is_empty() {
                    field.queue_edit(TextEdit::Insert(text.into()));
                }
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
