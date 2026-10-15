//! The current note's links, in a popover opened from the button beside its title in the top
//! bar: notes linking to it, and notes it links to. It closes on a click outside, on Escape
//! and when a note is opened.

use bevy::{
    prelude::*, text::FontWeight, ui::UiGlobalTransform, ui_widgets::Activate,
    window::PrimaryWindow,
};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::{
    Vault,
    nav::Nav,
    vault::{NoteId, VaultData},
};
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, place_panel.after(bevy::ui::UiSystems::Layout));
}

/// The button the popover opens from.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct LinksAnchor;

/// The popover.
#[derive(Component, Clone, Copy, Debug, Default)]
struct LinksPanel;

const WIDTH: f32 = 300.0;

/// The button: a link icon and how many links the note has (only the icon on phones).
#[component]
pub fn links_button(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>, compact: bool) -> impl Scene {
    let size = if compact { Size::IconLg } else { Size::Sm };
    let count = insert(font(size::SM, FontWeight::MEDIUM));
    view! {
        @{button(Variant::Ghost, size)}
        Node { padding: UiRect::horizontal(px(if compact { 0.0 } else { 8.0 })), flex_shrink: 0.0 }
        LinksAnchor
        Tooltip::new("Links")
        Selected($nav.links)
        on(|_: On<Activate>, mut n: ResMut<Nav>| n.links = !n.links)
        Children [
            @{tinted_icon("link", size.icon(), Tint::MUTED)}
            --
            if !compact {
                Text($({
                    let note = &$vault.notes[$nav.current];
                    (note.backlinks.len() + note.outgoing.len()).to_string()
                }))
                @{count}
                Tint::MUTED
                Pickable::IGNORE
            }
        ]
    }
}

/// The popover and the invisible layer under it that closes it on a click outside.
#[component]
pub fn links_popover(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>) -> impl Scene {
    let shadow = shadow::popover();
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::MENU)
        Pickable::IGNORE
        Children [
            if $nav.links {
                let shadow = shadow.clone();
                @{scrim(|world| world.resource_mut::<Nav>().links = false)}
                BackgroundColor(Color::NONE)
                Children [
                    @{popover()}
                    LinksPanel
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(WIDTH),
                        max_width: Val::Percent(94.0),
                        max_height: px(420),
                        overflow: Overflow::scroll_y(),
                    }
                    bevy::ui_widgets::ScrollArea
                    shadow
                    Children [
                        match ($nav.current, $vault.revision) {
                            (id, _) => { @{vault.with_untracked(|vault| lists(id, vault))} },
                        }
                    ]
                ]
            }
        ]
    }
}

/// "Backlinks" and "Outgoing links", each with its notes or a note that there are none.
fn lists(id: NoteId, data: &VaultData) -> Box<dyn Scene> {
    let note = &data.notes[id];
    let group = |title: &str, ids: &[NoteId], none: &str| -> Box<dyn Scene> {
        let rows: Vec<Box<dyn Scene>> = if ids.is_empty() {
            vec![Box::new(empty(none))]
        } else {
            ids.iter()
                .map(|&target| {
                    let linked = &data.notes[target];
                    Box::new(bsn! {
                        @{item()}
                        on(move |_: On<Activate>, mut n: ResMut<Nav>| n.open(target, None))
                        Children [ @{item_icon_char(linked.icon)} -- @{item_label(&linked.name)} ]
                    }) as Box<dyn Scene>
                })
                .collect()
        };
        let title = format!("{title} ({})", ids.len());
        Box::new(bsn! {
            Node { flex_direction: FlexDirection::Column }
            Children [ @{group_label(&title)} -- {rows} ]
        })
    };
    let backlinks = group("Backlinks", &note.backlinks, "No notes link here");
    let outgoing = group("Outgoing links", &note.outgoing, "This note links nowhere");
    Box::new(bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(4) }
        Children [ @{backlinks} -- @{menu_separator()} -- @{outgoing} ]
    })
}

/// Puts the popover under its button, inside the window.
fn place_panel(
    anchors: Query<(&ComputedNode, &UiGlobalTransform), With<LinksAnchor>>,
    mut panels: Query<&mut Node, With<LinksPanel>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(mut panel) = panels.single_mut() else {
        return;
    };
    // The visible button (the wide and phone bars each have one, but only one is shown).
    let Some((node, at)) = anchors.iter().find(|(node, _)| node.size.x > 0.0) else {
        return;
    };
    let scale = node.inverse_scale_factor;
    let (center, size) = (at.translation * scale, node.size * scale);
    let window_width = window.single().map_or(f32::MAX, Window::width);
    let left = (center.x - size.x / 2.0)
        .min(window_width - WIDTH - 8.0)
        .max(8.0);
    let top = center.y + size.y / 2.0 + 6.0;
    if panel.left != px(left) || panel.top != px(top) {
        panel.left = px(left);
        panel.top = px(top);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_spawns_the_panel() {
        let vault = Vault::new(VaultData::from_sources([
            ("A.md", "# A\n\n[[B]]"),
            ("B.md", "# B"),
        ]));
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy_radioactive::view::ViewPlugin,
        ));
        app.insert_resource(vault).insert_resource(Nav::new(0));
        app.world_mut().spawn_scene(links_popover()).unwrap();
        app.update();
        let panels = |app: &mut App| {
            app.world_mut()
                .query_filtered::<(), With<LinksPanel>>()
                .iter(app.world())
                .count()
        };
        assert_eq!(panels(&mut app), 0);
        app.world_mut().resource_mut::<Nav>().links = true;
        app.update();
        app.update();
        assert_eq!(panels(&mut app), 1);
    }
}
