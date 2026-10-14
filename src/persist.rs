//! Remembers the settings (text size, scroll speed, frame rate), the map's chosen routes and
//! the notes visited between visits: on the web in `localStorage`, natively not at all. Also
//! the storage behind saved note edits.

use bevy::prelude::*;

use bevsidian_core::{
    Vault,
    nav::{MAX_FPS_CHOICES, Nav, SCROLL_SPEEDS, TEXT_SIZES},
    vault::NoteId,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (load, load_map))
        .add_systems(Last, (save, save_map));
}

const TEXT_SIZE_KEY: &str = "bevsidian.textSize";
const SCROLL_SPEED_KEY: &str = "bevsidian.scrollSpeed";
const SHOW_FPS_KEY: &str = "bevsidian.showFps";
const MAX_FPS_KEY: &str = "bevsidian.maxFps";
const VSYNC_KEY: &str = "bevsidian.vsync";
const QUESTS_KEY: &str = "bevsidian.quests";
const WHOLE_MAP_KEY: &str = "bevsidian.wholeMap";
const VISITED_KEY: &str = "bevsidian.visited";

/// Notes by name, one per line (names survive notes being added or removed).
fn names(vault: &Vault, ids: &[NoteId]) -> String {
    let data = vault.get_untracked();
    ids.iter()
        .filter_map(|&id| data.notes.get(id).map(|n| n.name.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ids(vault: &Vault, names: &str) -> Vec<NoteId> {
    let data = vault.get_untracked();
    names.lines().filter_map(|name| data.find(name)).collect()
}

fn load_map(nav: Res<Nav>, vault: Res<Vault>) {
    if let Some(quests) = read(QUESTS_KEY) {
        nav.quests.set(Some(ids(&vault, &quests)));
    }
    if read(WHOLE_MAP_KEY).as_deref() == Some("1") {
        nav.whole_map.set(true);
    }
    if let Some(visited) = read(VISITED_KEY) {
        let mut visited = ids(&vault, &visited);
        let current = nav.current.get_untracked();
        if !visited.contains(&current) {
            visited.push(current);
        }
        nav.visited.set(visited);
    }
}

fn save_map(
    nav: Res<Nav>,
    vault: Res<Vault>,
    mut last: Local<Option<(Option<Vec<NoteId>>, bool, usize)>>,
) {
    let current = (
        nav.quests.get_untracked(),
        nav.whole_map.get_untracked(),
        nav.visited.with_untracked(Vec::len),
    );
    if last.as_ref() == Some(&current) {
        return;
    }
    // The first frame only records what `load_map` restored.
    if last.is_some() {
        if let Some(quests) = &current.0 {
            write(QUESTS_KEY, &names(&vault, quests));
        }
        write(WHOLE_MAP_KEY, if current.1 { "1" } else { "0" });
        write(VISITED_KEY, &names(&vault, &nav.visited.get_untracked()));
    }
    *last = Some(current);
}

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
