//! Remembers the settings (text size, scroll speed, frame rate) between visits: on the web
//! in `localStorage`, natively not at all. Also the storage behind saved note edits.

use bevy::prelude::*;

use bevsidian_core::nav::{MAX_FPS_CHOICES, Nav, SCROLL_SPEEDS, TEXT_SIZES};

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
