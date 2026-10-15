//! The settings dialog (`Ctrl+,` or Settings in the sidebar): reading and performance.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::nav::{MAX_FPS_CHOICES, Nav, SCROLL_SPEED_STEP, SCROLL_SPEEDS, TEXT_SIZES};
use bevsidian_ui::prelude::*;

/// Shown by the Help button.
const HELP: &str = "Bevsidian draws with WebGL, so it needs hardware acceleration. In Chrome or \
Edge, turn on \"Use graphics acceleration when available\" and restart the browser. On \
Android, chrome://gpu should say WebGL is \"Hardware accelerated\". Battery Saver limits the \
screen to 60 Hz.";

#[component]
pub fn settings_dialog(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DIALOG)
        Pickable::IGNORE
        Children [
            if $nav.settings {
                @{scrim(close)}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(10.0)) }
                Children [ @{panel()} ]
            }
        ]
    }
}

fn close(world: &mut World) {
    world.resource_mut::<Nav>().settings = false;
}

/// A field of `Nav` that a control shows and sets.
macro_rules! setting {
    ($nav:expr, $field:ident) => {
        Binding::resource($nav, |n: &Nav| n.$field, |n, value| n.$field = value)
    };
}

#[component]
fn panel(nav: Watch<Res<Nav>>, help: Watch<Local<bool>>) -> impl Scene {
    let text_size = stepper(setting!(nav.clone(), text_size), TEXT_SIZES, 1, |px| {
        format!("{px} px")
    });
    let scroll_speed = stepper(
        setting!(nav.clone(), scroll_speed),
        SCROLL_SPEEDS,
        SCROLL_SPEED_STEP.unsigned_abs(),
        |percent| format!("{}×", percent as f32 / 100.0),
    );
    let frame_cap = segmented(
        MAX_FPS_CHOICES
            .iter()
            .map(|&fps| {
                (
                    fps,
                    if fps == 0 {
                        "Off".to_owned()
                    } else {
                        fps.to_string()
                    },
                )
            })
            .collect(),
        setting!(nav.clone(), max_fps),
    );
    let (sound, show_fps) = (
        setting!(nav.clone(), sound),
        setting!(nav.clone(), show_fps),
    );
    let vsync = vsync(nav.clone());
    view! {
        @{dialog(480.0)}
        Children [
            @{dialog_header("Settings", close)}
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
            Children [
                @{field("Text size", None, text_size)}
                --
                @{field("Scroll speed", None, scroll_speed)}
                --
                @{field("Sound in examples", None, switch(sound))}
            ]
            --
            @{separator()}
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
            Children [
                @{field("Show FPS", None, switch(show_fps))}
                --
                @{field("Frame rate limit", None, frame_cap)}
                --
                @{field("VSync", None, vsync)}
            ]
            --
            if $help {
                @{alert(Tone::Default, "info", "Low frame rate or stutter?", HELP)}
            }
            --
            Node { align_items: AlignItems::Center, column_gap: px(8) }
            Children [
                Node { flex_grow: 1.0 }
                Children [
                    if !$nav.touch {
                        @{shortcuts(&[("Ctrl ,", "settings")])}
                    }
                ]
                --
                @{text_button(Variant::Outline, Size::Sm, "Help")}
                Selected($help)
                on(move |_: On<Activate>| help.update(|shown| *shown = !*shown))
                --
                @{text_button(Variant::Primary, Size::Sm, "Done")}
                on(|_: On<Activate>, mut n: ResMut<Nav>| n.settings = false)
            ]
        ]
    }
}

/// A switch on desktop. Browsers always present in sync with the display, so on the web this
/// shows the refresh rate the page is synced to instead.
fn vsync(nav: Watch<Res<'static, Nav>>) -> Box<dyn Scene> {
    if cfg!(target_arch = "wasm32") {
        Box::new(refresh_rate(nav))
    } else {
        Box::new(switch(setting!(nav, vsync)))
    }
}

fn refresh_rate(nav: Watch<Res<'static, Nav>>) -> impl Scene {
    let font = insert(font(size::SM, FontWeight::NORMAL));
    view! {
        Text($(match $nav.display_hz {
            Some(hz) => format!("{hz} Hz"),
            None => "Measuring…".to_owned(),
        }))
        @{font}
        TextColor(color::MUTED_FOREGROUND)
    }
}
