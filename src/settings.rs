//! Reading settings: text size, scroll speed and the frame-rate readout.
//!
//! Opened from the sliders button in the top bar (or `Ctrl+,`). On the web the values are
//! remembered in `localStorage`.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Signal, signal, view};

use bevsidian_core::nav::{MAX_FPS_CHOICES, Nav, SCROLL_SPEEDS, TEXT_SIZES};
use bevsidian_ui::{
    Selected, Size, Variant, color, dialog, field, icon_button, layer, note_box, scrim, separator,
    shortcuts, size, stepper, switch, text_button, title, ui_font,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, load).add_systems(Last, save);
}

const TEXT_SIZE_KEY: &str = "bevsidian.textSize";
const SCROLL_SPEED_KEY: &str = "bevsidian.scrollSpeed";
const SHOW_FPS_KEY: &str = "bevsidian.showFps";
const MAX_FPS_KEY: &str = "bevsidian.maxFps";
const VSYNC_KEY: &str = "bevsidian.vsync";

fn load(nav: Res<Nav>) {
    if let Some(size) = read(TEXT_SIZE_KEY).and_then(|v| v.parse::<u32>().ok()) {
        nav.text_size
            .set(size.clamp(*TEXT_SIZES.start(), *TEXT_SIZES.end()));
    }
    if let Some(speed) = read(SCROLL_SPEED_KEY).and_then(|v| v.parse::<u32>().ok()) {
        nav.scroll_speed
            .set(speed.clamp(*SCROLL_SPEEDS.start(), *SCROLL_SPEEDS.end()));
    }
    if let Some(cap) = read(MAX_FPS_KEY).and_then(|v| v.parse::<u32>().ok())
        && MAX_FPS_CHOICES.contains(&cap)
    {
        nav.max_fps.set(cap);
    }
    if read(VSYNC_KEY).as_deref() == Some("0") {
        nav.vsync.set(false);
    }
    let fps_requested = crate::perf::requested();
    if fps_requested || read(SHOW_FPS_KEY).as_deref() == Some("1") {
        nav.show_fps.set(true);
    }
}

fn save(nav: Res<Nav>, mut last: Local<Option<(u32, u32, bool, u32, bool)>>) {
    let current = (
        nav.text_size.get_untracked(),
        nav.scroll_speed.get_untracked(),
        nav.show_fps.get_untracked(),
        nav.max_fps.get_untracked(),
        nav.vsync.get_untracked(),
    );
    if *last == Some(current) {
        return;
    }
    // The first frame only records what `load` restored.
    if last.is_some() {
        write(TEXT_SIZE_KEY, &current.0.to_string());
        write(SCROLL_SPEED_KEY, &current.1.to_string());
        write(SHOW_FPS_KEY, if current.2 { "1" } else { "0" });
        write(MAX_FPS_KEY, &current.3.to_string());
        write(VSYNC_KEY, if current.4 { "1" } else { "0" });
    }
    *last = Some(current);
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub(crate) fn read(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        storage()?.get_item(key).ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        None
    }
}

pub(crate) fn write(key: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = storage() {
            // Private windows may refuse; the setting then lasts for this visit only.
            let _ = storage.set_item(key, value);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, value);
    }
}

/// `VSync` switch on desktop. In a browser the canvas is always presented in sync with the
/// display, so this shows the refresh rate it targets instead.
fn vsync_control(nav: &Nav) -> Box<dyn Scene> {
    if cfg!(target_arch = "wasm32") {
        Box::new(vsync_target(nav.display_hz.clone()))
    } else {
        Box::new(bevsidian_ui::switch(nav.vsync.clone()))
    }
}

/// "120 Hz target": the measured refresh rate the browser syncs the page to.
fn vsync_target(hz: Signal<Option<u32>>) -> impl Scene {
    view! {
        Text($(match $hz {
            Some(hz) => format!("{hz} Hz target"),
            None => "Measuring…".to_owned(),
        }))
        @{ui_font(size::SM, FontWeight::NORMAL)}
        TextColor(color::SUBTLE_FOREGROUND)
    }
}

/// Shown by the Help button.
const HELP: &str = "Low frame rate or stutter? Bevsidian draws with WebGL, so it needs \
hardware acceleration.\n\n\
• Desktop Chrome / Edge: Settings → System → turn on \"Use graphics acceleration when \
available\", then restart the browser.\n\
• Android: open chrome://gpu; WebGL should say \"Hardware accelerated\". If it does not, \
try another browser (Brave runs it at 120 Hz).\n\
• Battery Saver limits the screen to 60 Hz.";

pub fn settings_panel(nav: Nav) -> impl Scene {
    let is_open = nav.settings.clone();
    let help = signal(|| false);
    view! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        GlobalZIndex(layer::DIALOG)
        Pickable::IGNORE
        Children [
            if $is_open {
                let close = nav.settings.clone();
                @{scrim(move || close.set(false))}
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    padding: UiRect::top(Val::Percent(12.0)),
                }
                Children [ @{settings_dialog(nav.clone(), help.clone())} ]
            }
        ]
    }
}

fn settings_dialog(nav: Nav, help: Signal<bool>) -> impl Scene {
    let (close, done) = (nav.settings.clone(), nav.settings.clone());
    let text_size = {
        let (size, smaller, larger) = (nav.text_size.clone(), nav.clone(), nav.clone());
        stepper(
            move || format!("{} px", size.get()),
            move || smaller.change_text_size(-1),
            move || larger.change_text_size(1),
        )
    };
    let scroll_speed = {
        let (speed, slower, faster) = (nav.scroll_speed.clone(), nav.clone(), nav.clone());
        stepper(
            move || format!("{}×", speed.get() as f32 / 100.0),
            move || slower.change_scroll_speed(-1),
            move || faster.change_scroll_speed(1),
        )
    };
    let max_fps = {
        let (cap, fewer, more) = (nav.max_fps.clone(), nav.clone(), nav.clone());
        stepper(
            move || match cap.get() {
                0 => "No cap".to_owned(),
                cap => cap.to_string(),
            },
            move || fewer.change_max_fps(-1),
            move || more.change_max_fps(1),
        )
    };
    let toggle_help = help.clone();
    view! {
        @{dialog(360.0)}
        Children [
            Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center }
            Children [
                @{title("Settings")}
                --
                @{icon_button(Variant::Ghost, Size::IconSm, "x")}
                on(move |_: On<Activate>| close.set(false))
            ]
            --
            @{field("Text size", text_size)}
            --
            @{field("Scroll speed", scroll_speed)}
            --
            @{field("Show FPS", switch(nav.show_fps.clone()))}
            --
            @{field("Max FPS", max_fps)}
            --
            @{field("VSync", vsync_control(&nav))}
            --
            if $help {
                @{note_box(HELP)}
            }
            --
            @{separator()}
            --
            Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0) }
            Children [
                @{shortcuts(&[("Ctrl ,", "settings")])}
                Node { flex_grow: 1.0 }
                --
                @{text_button(Variant::Secondary, Size::Sm, "Help")}
                Selected($help)
                on(move |_: On<Activate>| toggle_help.update(|on| *on = !*on))
                --
                @{text_button(Variant::Primary, Size::Sm, "Done")}
                on(move |_: On<Activate>| done.set(false))
            ]
        ]
    }
}
