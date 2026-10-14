//! Everything that floats over the page: dialogs, menus, toolbars, toasts, the autoscroll
//! marker and the on-screen keyboard.

pub mod autoscroll;
pub mod command;
pub mod context_menu;
pub mod keyboard;
pub mod links;
pub mod selection_bar;
pub mod settings;
pub mod toast;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        autoscroll::plugin,
        context_menu::plugin,
        links::plugin,
        toast::plugin,
        selection_bar::plugin,
        keyboard::plugin,
    ));
}
