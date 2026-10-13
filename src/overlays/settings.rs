//! The settings dialog (`Ctrl+,` or Settings in the sidebar): reading and performance.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Signal, signal, view};

use bevsidian_core::nav::{MAX_FPS_CHOICES, Nav, SCROLL_SPEED_STEP, SCROLL_SPEEDS, TEXT_SIZES};
use bevsidian_ui::prelude::*;

/// Shown by the Help button.
const HELP: &str = "Bevsidian draws with WebGL, so it needs hardware acceleration. In Chrome or \
Edge, turn on \"Use graphics acceleration when available\" and restart the browser. On \
Android, chrome://gpu should say WebGL is \"Hardware accelerated\". Battery Saver limits the \
screen to 60 Hz.";

pub fn settings_dialog(nav: Nav) -> impl Scene {
    let open = nav.settings.clone();
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DIALOG)
        Pickable::IGNORE
        Children [
            if $open {
                let close = nav.settings.clone();
                @{scrim(move || close.set(false))}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(10.0)) }
                Children [ @{panel(nav.clone())} ]
            }
        ]
    }
}

fn panel(nav: Nav) -> impl Scene {
    let help = signal(|| false);
    let (close, done, toggle_help, touch) = (
        nav.settings.clone(),
        nav.settings.clone(),
        help.clone(),
        nav.touch.clone(),
    );
    let text_size = stepper(nav.text_size.clone(), TEXT_SIZES, 1, |px| {
        format!("{px} px")
    });
    let scroll_speed = stepper(
        nav.scroll_speed.clone(),
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
        nav.max_fps.clone(),
    );
    view! {
        @{dialog(480.0)}
        Children [
            @{dialog_header("Settings", move || close.set(false))}
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
            Children [
                @{field("Text size", None, text_size)}
                --
                @{field("Scroll speed", None, scroll_speed)}
            ]
            --
            @{separator()}
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
            Children [
                @{field("Show FPS", None, switch(nav.show_fps.clone()))}
                --
                @{field("Frame rate limit", None, frame_cap)}
                --
                @{field("VSync", None, vsync(&nav))}
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
                    if !$touch {
                        @{shortcuts(&[("Ctrl ,", "settings")])}
                    }
                ]
                --
                @{text_button(Variant::Outline, Size::Sm, "Help")}
                Selected($help)
                on(move |_: On<Activate>| toggle_help.update(|shown| *shown = !*shown))
                --
                @{text_button(Variant::Primary, Size::Sm, "Done")}
                on(move |_: On<Activate>| done.set(false))
            ]
        ]
    }
}

/// A switch on desktop. Browsers always present in sync with the display, so on the web this
/// shows the refresh rate the page is synced to instead.
fn vsync(nav: &Nav) -> Box<dyn Scene> {
    if cfg!(target_arch = "wasm32") {
        Box::new(refresh_rate(nav.display_hz.clone()))
    } else {
        Box::new(switch(nav.vsync.clone()))
    }
}

fn refresh_rate(hz: Signal<Option<u32>>) -> impl Scene {
    let font = insert(font(size::SM, FontWeight::NORMAL));
    view! {
        Text($(match $hz {
            Some(hz) => format!("{hz} Hz"),
            None => "Measuring…".to_owned(),
        }))
        @{font}
        TextColor(color::MUTED_FOREGROUND)
    }
}
