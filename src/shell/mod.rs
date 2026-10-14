//! The application shell: the sidebar, the top bar and the page under it, and the overlays
//! above everything.
//!
//! Static structure is plain BSN; whatever depends on navigation state is a `view!` region
//! driven by the [`Nav`] signals.

pub mod sidebar;
pub mod topbar;

use bevy::prelude::*;
use bevy_radioactive::view::view;
use bevy_ui_commonmark::MdTheme;

use bevsidian_core::{Vault, nav::Nav, nav::View};
use bevsidian_ui::color;

use crate::{overlays, pages};

pub fn plugin(app: &mut App) {
    app.add_plugins(sidebar::plugin)
        .add_systems(Startup, spawn_shell);
}

fn spawn_shell(mut commands: Commands, nav: Res<Nav>, vault: Res<Vault>, theme: Res<MdTheme>) {
    // No MSAA: UI is anti-aliased per glyph/shape, and 4x MSAA at phone resolutions is costly.
    commands.spawn((Camera2d, IsDefaultUiCamera, Msaa::Off));
    commands.spawn_scene(shell(nav.clone(), vault.clone(), theme.clone()));
}

fn shell(nav: Nav, vault: Vault, theme: MdTheme) -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            // UI roots are flex items of the camera viewport: without this the root grows to
            // the min-content width of wide content (e.g. long code lines) on narrow screens.
            min_width: px(0),
            min_height: px(0),
            overflow: Overflow::clip(),
        }
        BackgroundColor(color::BACKGROUND)
        Children [
            @{sidebar::sidebar(nav.clone(), vault.clone())}
            --
            @{main_column(nav.clone(), vault.clone(), theme)}
            --
            @{sidebar::backdrop(nav.clone())}
            --
            @{overlays::links::links_popover(nav.clone(), vault.clone())}
            --
            @{overlays::command::command_palette(nav.clone(), vault.clone())}
            --
            @{overlays::settings::settings_dialog(nav.clone())}
            --
            @{overlays::quests::quest_dialog(nav.clone(), vault)}
            --
            @{overlays::selection_bar::selection_bar(nav.clone())}
            --
            @{overlays::keyboard::keyboard(nav)}
        ]
    }
}

/// The top bar over the current page.
fn main_column(nav: Nav, vault: Vault, theme: MdTheme) -> impl Scene {
    let view = nav.view.clone();
    let top = topbar::topbar(nav.clone(), vault.clone());
    view! {
        Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip_x(),
        }
        Children [
            @{top}
            --
            match $view {
                View::Note => {
                    @{pages::note::note_page(nav.clone(), vault.clone(), theme.clone())}
                },
                View::Graph => {
                    @{pages::graph::graph_page(nav.clone(), vault.clone())}
                },
            }
        ]
    }
}
