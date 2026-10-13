//! The examples code blocks run, by name, and the UI scenes they show.
//!
//! To add an example: write its program in a file here (types, systems, observers), list its
//! functions in an [`Example`], and name it below. Notes then run it from any number of code
//! blocks with `example=NAME`; each block runs the functions its code mentions.

mod animation;
mod bouncing;
mod change;
mod commands;
mod dragging;
mod ecs;
mod events;
mod hierarchy;
mod messages;
mod movement;
mod queries;
mod resources;
pub mod scenes;
mod sprites;
mod stage;
mod tilemap;
mod timers;

use bevy::prelude::*;

use crate::example::Example;

/// Every example's name.
#[cfg(test)]
pub const NAMES: [&str; 15] = [
    "animation",
    "bouncing",
    "change",
    "commands",
    "dragging",
    "ecs",
    "events",
    "hierarchy",
    "messages",
    "movement",
    "queries",
    "resources",
    "sprites",
    "tilemap",
    "timers",
];

/// The example a code block names with `example=NAME`.
pub fn find(name: &str) -> Option<&'static Example> {
    Some(match name {
        "animation" => &animation::EXAMPLE,
        "bouncing" => &bouncing::EXAMPLE,
        "change" => &change::EXAMPLE,
        "commands" => &commands::EXAMPLE,
        "dragging" => &dragging::EXAMPLE,
        "ecs" => &ecs::EXAMPLE,
        "events" => &events::EXAMPLE,
        "hierarchy" => &hierarchy::EXAMPLE,
        "messages" => &messages::EXAMPLE,
        "movement" => &movement::EXAMPLE,
        "queries" => &queries::EXAMPLE,
        "resources" => &resources::EXAMPLE,
        "sprites" => &sprites::EXAMPLE,
        "tilemap" => &tilemap::EXAMPLE,
        "timers" => &timers::EXAMPLE,
        _ => return None,
    })
}

/// Assets the examples share with their previews.
pub fn plugin(app: &mut App) {
    app.add_plugins(animation::plugin);
}
