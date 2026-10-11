//! Bevsidian: an Obsidian-like documentation vault for Bevy, built with Bevy.
//!
//! Markdown notes from `vault/` are embedded at compile time, rendered with
//! `bevy_ui_commonmark`, selectable with `bevy_ui_textselect`, and navigated through
//! `[[wikilinks]]`, `#tags`, backlinks and a graph view.

mod graph;
mod nav;
mod ui;
mod vault;

use std::sync::Arc;

use bevy::{feathers::FeathersPlugins, prelude::*, text::Font};
use bevy_radioactive::RadioactivePlugin;
use bevy_ui_commonmark::CommonMarkPlugin;

use crate::{
    nav::Nav,
    vault::{Vault, VaultData},
};

fn main() {
    let vault = Vault(Arc::new(VaultData::embedded()));
    let nav = Nav::new(vault.home);
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
        .insert_resource(ClearColor(ui::palette::BG))
        .insert_resource(vault)
        .insert_resource(nav)
        .add_systems(PreStartup, load_fonts)
        .add_plugins((nav::plugin, ui::plugin, graph::plugin))
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
    ];
    commands.insert_resource(EmbeddedFonts(handles.to_vec()));
}

/// Keeps the embedded font assets alive.
#[derive(Resource)]
#[allow(dead_code)]
struct EmbeddedFonts(Vec<Handle<Font>>);
