//! Remembers the settings (text size, scroll speed, frame rate), the map's chosen routes, the
//! notes visited and the reader's own sprite, voxel model and sounds ("Make it yours") between
//! visits: on the web in `localStorage`, natively in a file per setting in the user's data
//! folder. Also the storage behind saved note edits on the web.

use bevy::prelude::*;

use bevsidian_preview::{
    sound::{Sfx, Slot},
    yours::{self, SPRITE, VOXELS, Yours},
};

use bevsidian_core::{
    Vault,
    nav::{MAX_FPS_CHOICES, Nav, SCROLL_SPEEDS, TEXT_SIZES},
    vault::NoteId,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (load, load_map, load_yours))
        .add_systems(Last, (save, save_map, save_yours));
}

const TEXT_SIZE_KEY: &str = "bevsidian.textSize";
const SCROLL_SPEED_KEY: &str = "bevsidian.scrollSpeed";
const SHOW_FPS_KEY: &str = "bevsidian.showFps";
const MAX_FPS_KEY: &str = "bevsidian.maxFps";
const VSYNC_KEY: &str = "bevsidian.vsync";
const QUESTS_KEY: &str = "bevsidian.quests";
const WHOLE_MAP_KEY: &str = "bevsidian.wholeMap";
const VISITED_KEY: &str = "bevsidian.visited";
const SPRITE_KEY: &str = "bevsidian.sprite";
const VOXELS_KEY: &str = "bevsidian.voxels";
const SOUNDS_KEY: &str = "bevsidian.sounds";
const SOUND_KEY: &str = "bevsidian.sound";

/// The reader's sprite and model, if they made them (an empty value: they started over).
fn load_yours(mut yours: ResMut<Yours>) {
    if let Some(pixels) = read(SPRITE_KEY).and_then(|v| yours::decode(&v, SPRITE * SPRITE)) {
        yours.pixels = pixels;
        yours.made_sprite = true;
    }
    if let Some(voxels) = read(VOXELS_KEY).and_then(|v| yours::decode(&v, VOXELS * VOXELS * VOXELS))
    {
        yours.voxels = voxels;
        yours.made_voxels = true;
    }
    // One sound per line, in slot order.
    if let Some(text) = read(SOUNDS_KEY) {
        let sounds: Option<Vec<Sfx>> = text.lines().map(Sfx::decode).collect();
        if let Some(sounds) = sounds.filter(|s| s.len() == Slot::ALL.len()) {
            yours.sounds = sounds;
            yours.made_sounds = true;
        }
    }
}

fn save_yours(yours: Res<Yours>, mut last: Local<Option<(String, String, String)>>) {
    if !yours.is_changed() {
        return;
    }
    let stored = |made: bool, cells: &[yours::Cell]| {
        if made {
            yours::encode(cells)
        } else {
            String::new()
        }
    };
    let sounds = if yours.made_sounds {
        yours
            .sounds
            .iter()
            .map(Sfx::encode)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };
    let current = (
        stored(yours.made_sprite, &yours.pixels),
        stored(yours.made_voxels, &yours.voxels),
        sounds,
    );
    if last.as_ref() == Some(&current) {
        return;
    }
    // The first change only records what `load_yours` restored.
    if last.is_some() {
        write(SPRITE_KEY, &current.0);
        write(VOXELS_KEY, &current.1);
        write(SOUNDS_KEY, &current.2);
    }
    *last = Some(current);
}

/// Notes by name, one per line (names survive notes being added or removed).
fn names(vault: &Vault, ids: &[NoteId]) -> String {
    ids.iter()
        .filter_map(|&id| vault.notes.get(id).map(|n| n.name.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ids(vault: &Vault, names: &str) -> Vec<NoteId> {
    names.lines().filter_map(|name| vault.find(name)).collect()
}

fn load_map(mut nav: ResMut<Nav>, vault: Res<Vault>) {
    if let Some(quests) = read(QUESTS_KEY) {
        nav.quests = Some(ids(&vault, &quests));
    }
    if read(WHOLE_MAP_KEY).as_deref() == Some("1") {
        nav.whole_map = true;
    }
    if let Some(visited) = read(VISITED_KEY) {
        nav.visited = ids(&vault, &visited);
    }
}

fn save_map(
    nav: Res<Nav>,
    vault: Res<Vault>,
    mut last: Local<Option<(Option<Vec<NoteId>>, bool, usize)>>,
) {
    if !nav.is_changed() {
        return;
    }
    let current = (nav.quests.clone(), nav.whole_map, nav.visited.len());
    if last.as_ref() == Some(&current) {
        return;
    }
    // The first frame only records what `load_map` restored.
    if last.is_some() {
        if let Some(quests) = &current.0 {
            write(QUESTS_KEY, &names(&vault, quests));
        }
        write(WHOLE_MAP_KEY, if current.1 { "1" } else { "0" });
        write(VISITED_KEY, &names(&vault, &nav.visited));
    }
    *last = Some(current);
}

fn load(mut nav: ResMut<Nav>) {
    if let Some(size) = read(TEXT_SIZE_KEY).and_then(|v| v.parse::<u32>().ok()) {
        nav.text_size = size.clamp(*TEXT_SIZES.start(), *TEXT_SIZES.end());
    }
    if let Some(speed) = read(SCROLL_SPEED_KEY).and_then(|v| v.parse::<u32>().ok()) {
        nav.scroll_speed = speed.clamp(*SCROLL_SPEEDS.start(), *SCROLL_SPEEDS.end());
    }
    if let Some(cap) = read(MAX_FPS_KEY).and_then(|v| v.parse::<u32>().ok())
        && MAX_FPS_CHOICES.contains(&cap)
    {
        nav.max_fps = cap;
    }
    if read(VSYNC_KEY).as_deref() == Some("0") {
        nav.vsync = false;
    }
    if read(SOUND_KEY).as_deref() == Some("0") {
        nav.sound = false;
    }
    let fps_requested = crate::perf::requested();
    if fps_requested || read(SHOW_FPS_KEY).as_deref() == Some("1") {
        nav.show_fps = true;
    }
}

fn save(nav: Res<Nav>, mut last: Local<Option<(u32, u32, bool, u32, bool, bool)>>) {
    if !nav.is_changed() {
        return;
    }
    let current = (
        nav.text_size,
        nav.scroll_speed,
        nav.show_fps,
        nav.max_fps,
        nav.vsync,
        nav.sound,
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
        write(SOUND_KEY, if current.5 { "1" } else { "0" });
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
        std::fs::read_to_string(native_file(key)?).ok()
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
        // A setting that cannot be written lasts for this run only.
        if let Some(file) = native_file(key) {
            let _ = file
                .parent()
                .map(std::fs::create_dir_all)
                .transpose()
                .and_then(|_| std::fs::write(file, value));
        }
    }
}

/// Where a setting lives natively: a file named after its key in the user's data folder
/// (`$XDG_DATA_HOME/bevsidian`, `~/.local/share/bevsidian`, or `%APPDATA%\bevsidian`).
#[cfg(not(target_arch = "wasm32"))]
fn native_file(key: &str) -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::Path::new(&home).join(".local/share"))
        })
        .or_else(|| std::env::var_os("APPDATA").map(std::path::PathBuf::from))?;
    let name: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    Some(base.join("bevsidian").join(name))
}
