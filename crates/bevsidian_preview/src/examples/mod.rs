//! The examples code blocks run, by name, and the UI scenes they show.
//!
//! To add an example: write its program in a file here (types, systems, observers), list its
//! functions in an [`Example`], and name it below. Notes then run it from any number of code
//! blocks with `example=NAME`; each block runs the functions its code mentions.

mod ai;
mod animation;
mod bouncing;
mod cave;
mod change;
mod commands;
mod conditions;
mod dodge;
mod dragging;
mod ecs;
mod events;
mod feel;
mod hierarchy;
mod messages;
mod movement;
mod pathfinding;
mod queries;
mod resources;
pub mod scenes;
mod sounds;
mod sprites;
mod stage;
mod states;
mod tilemap;
mod timers;
mod voxels;

use bevy::prelude::*;

use crate::example::Example;

/// Every example's name.
#[cfg(test)]
pub const NAMES: [&str; 24] = [
    "ai",
    "animation",
    "bouncing",
    "cave",
    "change",
    "commands",
    "conditions",
    "dodge",
    "dragging",
    "ecs",
    "events",
    "feel",
    "hierarchy",
    "messages",
    "movement",
    "pathfinding",
    "queries",
    "resources",
    "sounds",
    "sprites",
    "states",
    "tilemap",
    "timers",
    "voxels",
];

/// The example a code block names with `example=NAME`.
pub fn find(name: &str) -> Option<&'static Example> {
    Some(match name {
        "ai" => &ai::EXAMPLE,
        "animation" => &animation::EXAMPLE,
        "bouncing" => &bouncing::EXAMPLE,
        "cave" => &cave::EXAMPLE,
        "change" => &change::EXAMPLE,
        "commands" => &commands::EXAMPLE,
        "conditions" => &conditions::EXAMPLE,
        "dodge" => &dodge::EXAMPLE,
        "dragging" => &dragging::EXAMPLE,
        "ecs" => &ecs::EXAMPLE,
        "events" => &events::EXAMPLE,
        "feel" => &feel::EXAMPLE,
        "hierarchy" => &hierarchy::EXAMPLE,
        "messages" => &messages::EXAMPLE,
        "movement" => &movement::EXAMPLE,
        "pathfinding" => &pathfinding::EXAMPLE,
        "queries" => &queries::EXAMPLE,
        "resources" => &resources::EXAMPLE,
        "sounds" => &sounds::EXAMPLE,
        "sprites" => &sprites::EXAMPLE,
        "states" => &states::EXAMPLE,
        "tilemap" => &tilemap::EXAMPLE,
        "timers" => &timers::EXAMPLE,
        "voxels" => &voxels::EXAMPLE,
        _ => return None,
    })
}

/// Assets the examples share with their previews.
pub fn plugin(app: &mut App) {
    app.add_plugins(animation::plugin)
        .add_systems(Update, scenes::maths::animate_easing);
}
