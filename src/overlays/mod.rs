//! Everything that floats over the page: dialogs (settings, the map's checklist), menus,
//! toolbars, toasts, find in the note, the autoscroll marker and the on-screen keyboard.

pub mod autoscroll;
pub mod command;
pub mod context_menu;
pub mod find;
pub mod keyboard;
pub mod links;
pub mod quests;
pub mod selection_bar;
pub mod settings;
pub mod toast;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        autoscroll::plugin,
        context_menu::plugin,
        find::plugin,
        links::plugin,
        quests::plugin,
        toast::plugin,
        selection_bar::plugin,
        keyboard::plugin,
    ));
}
