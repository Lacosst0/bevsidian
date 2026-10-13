//! Responsive layout and touch support.
//!
//! - [`LayoutMode`] follows the window width: the sidebar is inline on wide screens and a
//!   slide-over drawer on phones.
//! - Touch drags scroll and fling ([`bevy_ui_touchscroll`]), at the scroll speed setting.
//! - The selection toolbar and the on-screen keyboard for touch screens live in
//!   [`overlays`](crate::overlays).

use bevy::{
    input::{
        mouse::MouseButtonInput,
        touch::{TouchInput, Touches},
    },
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    window::PrimaryWindow,
};
use std::time::Duration;

use bevy_ui_textselect::TextSelection;
use bevy_ui_touchscroll::{ScrollSpeed, SmoothScroll, TouchScroll, TouchScrollPlugin};

use bevsidian_core::{Nav, View, nav::LayoutMode};

pub fn plugin(app: &mut App) {
    app.add_plugins(TouchScrollPlugin)
        .add_systems(
            Update,
            (update_layout_mode, detect_touch, sync_scroll_speed),
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
    previews: Query<&bevsidian_preview::PreviewRuntime>,
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
        || smooth.iter().any(SmoothScroll::animating)
        || previews
            .iter()
            .any(bevsidian_preview::PreviewRuntime::is_playing);
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
        nav.drawer.set_if_neq(false);
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
