//! The examples code blocks run, by name, and the UI scenes they show.
//!
//! To add an example: write its program in a file here (types, systems, observers), list its
//! functions in an [`Example`], and name it below. Notes then run it from any number of code
//! blocks with `example=NAME`; each block runs the functions its code mentions.

mod ecs;
mod mario;
pub mod scenes;

use bevy::prelude::*;

use crate::example::Example;

/// The example a code block names with `example=NAME`.
pub fn find(name: &str) -> Option<&'static Example> {
    Some(match name {
        "ecs" => &ecs::EXAMPLE,
        "mario" => &mario::EXAMPLE,
        _ => return None,
    })
}

/// Assets the examples share with their previews.
pub fn plugin(app: &mut App) {
    app.add_plugins(mario::plugin);
}
