//! Bevsidian: an Obsidian-like documentation vault for Bevy, built with Bevy.
//!
//! Markdown notes from `vault/` are embedded at compile time, rendered with
//! `bevy_ui_commonmark`, selectable with `bevy_ui_textselect`, and navigated through
//! `[[wikilinks]]`, `#tags`, backlinks and a graph view.

mod editor;
mod layout;
mod menu;
mod perf;
mod settings;
mod ui;

use std::time::Duration;

use bevy::{
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    prelude::*,
    text::Font,
    winit::{UpdateMode, WinitSettings},
};
use bevy_radioactive::RadioactivePlugin;
use bevy_ui_commonmark::CommonMarkPlugin;

use bevsidian_core::{Nav, Vault, VaultData, theme::palette};

include!(concat!(env!("OUT_DIR"), "/notes.rs"));

fn main() {
    let notes = editor::load_edits(NOTES);
    let vault = Vault::new(VaultData::from_sources(
        notes
            .iter()
            .map(|(path, source)| (path.as_str(), source.as_str())),
    ));
    let nav = Nav::new(vault.get_untracked().home);
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevsidian".into(),
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((FeathersPlugins, RadioactivePlugin, CommonMarkPlugin))
        .insert_resource(ClearColor(palette::BG))
        // Render on input and while something animates (see `layout::keep_awake`).
        .insert_resource(WinitSettings {
            focused_mode: layout::IDLE,
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(5)),
        })
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(vault)
        .insert_resource(nav)
        .add_systems(PreStartup, load_fonts)
        .add_plugins((
            bevsidian_core::nav::plugin,
            ui::plugin,
            bevsidian_graph::plugin,
            layout::plugin,
            menu::plugin,
            perf::plugin,
            settings::plugin,
            editor::plugin,
        ))
        .run();
}

/// Embeds the UI fonts so the web build needs no asset folder.
fn load_fonts(mut fonts: ResMut<Assets<Font>>, mut commands: Commands) {
    let handles = [
        fonts.add(Font::from_bytes(
            include_bytes!("../assets/fonts/Inter.ttf").to_vec(),
        )),
        fonts.add(Font::from_bytes(
            include_bytes!("../assets/fonts/Inter-Italic.ttf").to_vec(),
        )),
        fonts.add(Font::from_bytes(
            include_bytes!("../assets/fonts/JetBrainsMono.ttf").to_vec(),
        )),
        // Note icons (a Lucide subset, see `tools/icons.py`).
        fonts.add(Font::from_bytes(
            include_bytes!("../assets/fonts/Lucide.ttf").to_vec(),
        )),
    ];
    commands.insert_resource(EmbeddedFonts(handles.to_vec()));
}

/// Keeps the embedded font assets alive.
#[derive(Resource)]
#[allow(dead_code)]
struct EmbeddedFonts(Vec<Handle<Font>>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_vault_is_consistent() {
        let v = VaultData::from_sources(NOTES.iter().copied());
        assert!(v.notes.len() >= 4);
        for note in &v.notes {
            assert!(
                note.unresolved.is_empty(),
                "{} has broken links: {:?}",
                note.name,
                note.unresolved
            );
            if let Some((_, icon)) = note.doc.properties.iter().find(|(k, _)| k == "icon") {
                assert!(
                    bevsidian_core::icons::find(icon).is_some(),
                    "{} has an unknown icon {icon:?} (add it to tools/icons.py)",
                    note.name
                );
            }
        }
    }
}
