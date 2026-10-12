//! Bevsidian: an Obsidian-like documentation vault for Bevy, built with Bevy.
//!
//! Markdown notes from `vault/` are embedded at compile time, rendered with
//! `bevy_ui_commonmark`, selectable with `bevy_ui_textselect`, and navigated through
//! `[[wikilinks]]`, `#tags`, backlinks and a graph view.

mod graph;
mod layout;
mod menu;
mod nav;
mod perf;
mod settings;
mod ui;
mod vault;

use std::{sync::Arc, time::Duration};

use bevy::{
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    prelude::*,
    text::Font,
    winit::{UpdateMode, WinitSettings},
};
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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevsidian".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                // Plugins this app does not use. Each still runs systems every frame, which
                // adds up on a single-threaded phone browser.
                .disable::<bevy::gilrs::GilrsPlugin>()
                .disable::<bevy::gizmos::GizmoPlugin>()
                .disable::<bevy::gizmos_render::GizmoRenderPlugin>()
                .disable::<bevy::post_process::PostProcessPlugin>()
                .disable::<bevy::animation::AnimationPlugin>()
                .disable::<bevy::state::app::StatesPlugin>()
                .disable::<bevy::world_serialization::WorldSerializationPlugin>(),
        )
        .add_plugins((FeathersPlugins, RadioactivePlugin, CommonMarkPlugin))
        .insert_resource(ClearColor(ui::palette::BG))
        // Render on input and while something animates (see `layout::keep_awake`).
        .insert_resource(WinitSettings {
            focused_mode: layout::IDLE,
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(5)),
        })
        // Nothing runs in `FixedUpdate`; without this the fixed-step loop still runs its
        // schedules (up to several times per frame on slow frames).
        .insert_resource(Time::<Fixed>::from_seconds(3600.0))
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(vault)
        .insert_resource(nav)
        .add_systems(PreStartup, load_fonts)
        .add_plugins((
            nav::plugin,
            ui::plugin,
            graph::plugin,
            layout::plugin,
            menu::plugin,
            perf::plugin,
            settings::plugin,
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
    ];
    commands.insert_resource(EmbeddedFonts(handles.to_vec()));
}

/// Keeps the embedded font assets alive.
#[derive(Resource)]
#[allow(dead_code)]
struct EmbeddedFonts(Vec<Handle<Font>>);
