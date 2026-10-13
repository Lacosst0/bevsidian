//! Everything that floats over the page: dialogs, menus, toolbars, toasts and the on-screen
//! keyboard.

pub mod command;
pub mod context_menu;
pub mod keyboard;
pub mod selection_bar;
pub mod settings;
pub mod toast;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        context_menu::plugin,
        toast::plugin,
        selection_bar::plugin,
        keyboard::plugin,
    ));
}
