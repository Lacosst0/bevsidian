//! The application shell: the sidebar, the top bar and the page under it, and the overlays
//! above everything.
//!
//! Static structure is plain BSN; whatever depends on navigation state is a `#[component]`
//! that watches the [`Nav`] resource and reads it in `view!` regions.

pub mod sidebar;
pub mod topbar;

use bevy::prelude::*;
use bevy_radioactive::view::{Watch, component, view};
use bevy_ui_commonmark::MdTheme;

use bevsidian_core::{nav::Nav, nav::View};
use bevsidian_ui::color;

use crate::{overlays, pages};

pub fn plugin(app: &mut App) {
    app.add_plugins(sidebar::plugin)
        .add_systems(Startup, spawn_shell);
}

fn spawn_shell(mut commands: Commands, theme: Res<MdTheme>) {
    // No MSAA: UI is anti-aliased per glyph/shape, and 4x MSAA at phone resolutions is costly.
    commands.spawn((Camera2d, IsDefaultUiCamera, Msaa::Off));
    commands.spawn_scene(shell(theme.clone()));
}

fn shell(theme: MdTheme) -> impl Scene {
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
            @{sidebar::sidebar()}
            --
            @{main_column(theme)}
            --
            @{sidebar::backdrop()}
            --
            @{overlays::links::links_popover()}
            --
            @{overlays::command::command_palette()}
            --
            @{overlays::settings::settings_dialog()}
            --
            @{overlays::quests::quest_dialog()}
            --
            @{overlays::studio::studio_dialog()}
            --
            @{overlays::selection_bar::selection_bar()}
            --
            @{overlays::keyboard::keyboard()}
        ]
    }
}

/// The top bar over the current page.
#[component]
fn main_column(nav: Watch<Res<Nav>>, theme: MdTheme) -> impl Scene {
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
            @{topbar::topbar()}
            --
            match $nav.view {
                View::Note => {
                    @{pages::note::note_page(theme.clone())}
                },
                View::Graph => {
                    // On phones the quest log starts folded, out of the map's way.
                    @{pages::graph::graph_page(nav.with_untracked(|nav| nav.mode.drawer()))}
                },
            }
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevsidian_core::{Vault, VaultData, nav::Studio};

    /// Watches can only be read once their scene has spawned: spawn the whole shell, then open
    /// every overlay, editor tab and view, so a read too early panics here rather than in the
    /// browser.
    #[test]
    fn every_screen_spawns() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy_radioactive::view::ViewPlugin,
        ))
        .insert_resource(Vault::new(VaultData::from_sources([
            ("A.md", "# A\n## Part\n[[Tour]] #tag"),
            ("Tour.md", "---\npath: true\n---\n# Tour\n1. [[A]]"),
        ])))
        .insert_resource(Nav::new(0))
        .init_resource::<bevsidian_preview::yours::Yours>()
        .init_resource::<overlays::studio::SoundSlot>();
        app.world_mut()
            .spawn_scene(shell(MdTheme::default()))
            .unwrap();
        app.update();
        for (view, studio) in [
            (View::Note, Studio::Sprite),
            (View::Graph, Studio::Voxels),
            (View::Note, Studio::Sounds),
        ] {
            let mut nav = app.world_mut().resource_mut::<Nav>();
            nav.view = view;
            nav.studio = Some(studio);
            nav.switcher = true;
            nav.settings = true;
            nav.map_setup = true;
            nav.links = true;
            nav.find = true;
            nav.drawer = true;
            nav.path = Some(1);
            nav.query = "a".into();
            app.update();
            app.update();
        }
    }
}
