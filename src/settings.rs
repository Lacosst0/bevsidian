//! Reading settings: text size, scroll speed and the frame-rate readout.
//!
//! Opened from the sliders button in the top bar (or `Ctrl+,`). On the web the values are
//! remembered in `localStorage`.

use bevy::{picking::events::PointerClick, prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::coat::{Signal, coat};

use crate::{
    nav::{MAX_FPS_CHOICES, Nav, SCROLL_SPEEDS, TEXT_SIZES},
    ui::{palette, segment, ui_font},
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

fn read(key: &str) -> Option<String> {
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

fn write(key: &str, value: &str) {
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

/// A "sliders" icon drawn with nodes.
pub fn sliders_icon() -> impl Scene {
    fn track(knob: f32) -> impl Scene {
        bsn! {
            Node {
                width: Val::Px(16.0),
                height: Val::Px(2.0),
                border_radius: BorderRadius::all(Val::Px(1.0)),
            }
            BackgroundColor(palette::TEXT)
            Pickable::IGNORE
            Children [
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(knob),
                    top: Val::Px(-2.0),
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                }
                BackgroundColor(palette::TEXT)
                Pickable::IGNORE
            ]
        }
    }
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(5.0) }
        Pickable::IGNORE
        Children [ @{track(9.0)} -- @{track(2.0)} -- @{track(7.0)} ]
    }
}

/// One labelled row of the panel.
fn row_label(label: &str) -> impl Scene + use<> {
    let text = Text::new(label);
    bsn! {
        text @{ui_font(14.0, FontWeight::NORMAL)} TextColor(palette::TEXT)
        Node { flex_grow: 1.0 }
    }
}

/// An on/off switch bound to `value`.
fn switch(value: Signal<bool>) -> impl Scene {
    let toggle = value.clone();
    coat! {
        Node {
            width: Val::Px(44.0),
            height: Val::Px(24.0),
            padding: UiRect::all(Val::Px(3.0)),
            border_radius: BorderRadius::all(Val::Px(12.0)),
            justify_content: $(if $value { JustifyContent::End } else { JustifyContent::Start }),
        }
        BackgroundColor($(if $value { palette::ACCENT } else { palette::BG_HOVER }))
        bevy::ui_widgets::Button
        bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
        on(move |_: On<Activate>| toggle.set(!toggle.get_untracked()))
        Children [
            Node {
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                border_radius: BorderRadius::all(Val::Px(9.0)),
            }
            BackgroundColor(palette::TEXT)
            Pickable::IGNORE
        ]
    }
}

/// `VSync` switch on desktop. In a browser the canvas is always presented in sync with the
/// display, so this shows the refresh rate it targets instead.
fn vsync_control(nav: &Nav) -> Box<dyn Scene> {
    if cfg!(target_arch = "wasm32") {
        Box::new(vsync_target(nav.display_hz.clone()))
    } else {
        Box::new(switch(nav.vsync.clone()))
    }
}

/// "120 Hz target": the measured refresh rate the browser syncs the page to.
fn vsync_target(hz: Signal<Option<u32>>) -> impl Scene {
    coat! {
        Text($(match $hz {
            Some(hz) => format!("{hz} Hz target"),
            None => "Measuring…".to_owned(),
        }))
        @{ui_font(12.5, FontWeight::NORMAL)}
        TextColor(palette::FAINT)
    }
}

/// The settings dialog.
pub fn settings_panel(nav: Nav) -> impl Scene {
    let (is_open, size, speed, max_fps) = (
        nav.settings.clone(),
        nav.text_size.clone(),
        nav.scroll_speed.clone(),
        nav.max_fps.clone(),
    );
    coat! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        GlobalZIndex(110)
        Pickable::IGNORE
        Children [
            if $is_open {
                let (backdrop, done) = (nav.clone(), nav.clone());
                let (smaller, larger, slower, faster, toggle_fps) =
                    (nav.clone(), nav.clone(), nav.clone(), nav.clone(), nav.clone());
                let (fewer, more, vsync_nav) = (nav.clone(), nav.clone(), nav.clone());
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    padding: UiRect::top(Val::Percent(12.0)),
                }
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55))
                on(move |click: On<PointerClick>| {
                    if click.entity == click.original_event_target() {
                        backdrop.settings.set(false);
                    }
                })
                Children [
                    Node {
                        width: Val::Px(360.0),
                        max_width: Val::Percent(92.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(14.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                    }
                    BackgroundColor(palette::BG_SIDEBAR)
                    BorderColor::all(palette::BORDER)
                    BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.5), Val::Px(0.0), Val::Px(8.0), Val::Px(0.0), Val::Px(24.0))
                    Children [
                        Text("Settings") @{ui_font(16.0, FontWeight::SEMIBOLD)} TextColor(palette::TEXT)
                        --
                        Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0) }
                        Children [
                            @{row_label("Text size")}
                            --
                            @{segment("A−")}
                            BackgroundColor(palette::BG_HOVER)
                            on(move |_: On<Activate>| smaller.change_text_size(-1))
                            --
                            Text($(format!("{} px", $size))) @{ui_font(13.0, FontWeight::NORMAL)} TextColor(palette::MUTED)
                            Node { min_width: Val::Px(52.0), justify_content: JustifyContent::Center }
                            --
                            @{segment("A+")}
                            BackgroundColor(palette::BG_HOVER)
                            on(move |_: On<Activate>| larger.change_text_size(1))
                        ]
                        --
                        Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0) }
                        Children [
                            @{row_label("Scroll speed")}
                            --
                            @{segment("−")}
                            BackgroundColor(palette::BG_HOVER)
                            on(move |_: On<Activate>| slower.change_scroll_speed(-1))
                            --
                            Text($(format!("{}×", $speed as f32 / 100.0))) @{ui_font(13.0, FontWeight::NORMAL)} TextColor(palette::MUTED)
                            Node { min_width: Val::Px(52.0), justify_content: JustifyContent::Center }
                            --
                            @{segment("+")}
                            BackgroundColor(palette::BG_HOVER)
                            on(move |_: On<Activate>| faster.change_scroll_speed(1))
                        ]
                        --
                        Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0) }
                        Children [
                            @{row_label("Show FPS")}
                            --
                            @{switch(toggle_fps.show_fps.clone())}
                        ]
                        --
                        Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0) }
                        Children [
                            @{row_label("Max FPS")}
                            --
                            @{segment("−")}
                            BackgroundColor(palette::BG_HOVER)
                            on(move |_: On<Activate>| fewer.change_max_fps(-1))
                            --
                            Text($(if $max_fps == 0 { "No cap".to_owned() } else { $max_fps.to_string() }))
                            @{ui_font(13.0, FontWeight::NORMAL)} TextColor(palette::MUTED)
                            Node { min_width: Val::Px(52.0), justify_content: JustifyContent::Center }
                            --
                            @{segment("+")}
                            BackgroundColor(palette::BG_HOVER)
                            on(move |_: On<Activate>| more.change_max_fps(1))
                        ]
                        --
                        Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0) }
                        Children [
                            @{row_label("VSync")}
                            --
                            @{vsync_control(&vsync_nav)}
                        ]
                        --
                        Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center }
                        Children [
                            Text("Ctrl+, opens settings") @{ui_font(11.5, FontWeight::NORMAL)} TextColor(palette::FAINT)
                            --
                            @{segment("Done")}
                            BackgroundColor(palette::BG_ACTIVE)
                            on(move |_: On<Activate>| done.settings.set(false))
                        ]
                    ]
                ]
            }
        ]
    }
}
