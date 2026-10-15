//! The current note's links, in a popover opened from the button beside its title in the top
//! bar: notes linking to it, and notes it links to. It closes on a click outside, on Escape
//! and when a note is opened.

use bevsidian_core::signal::memo;
use bevy::{
    prelude::*, text::FontWeight, ui::UiGlobalTransform, ui_widgets::Activate,
    window::PrimaryWindow,
};
use bevy_radioactive::view::view;

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
pub fn links_button(nav: Nav, vault: Vault, compact: bool) -> impl Scene {
    let (open, toggle, current) = (nav.links.clone(), nav.links.clone(), nav.current.clone());
    let size = if compact { Size::IconLg } else { Size::Sm };
    let count = insert(font(size::SM, FontWeight::MEDIUM));
    view! {
        @{button(Variant::Ghost, size)}
        Node { padding: UiRect::horizontal(px(if compact { 0.0 } else { 8.0 })), flex_shrink: 0.0 }
        LinksAnchor
        Tooltip::new("Links")
        Selected($open)
        on(move |_: On<Activate>| toggle.update(|shown| *shown = !*shown))
        Children [
            @{tinted_icon("link", size.icon(), Tint::MUTED)}
            --
            if !compact {
                Text($({
                    let note = &vault.get().notes[$current];
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
pub fn links_popover(nav: Nav, vault: Vault) -> impl Scene {
    let open = nav.links.clone();
    let key = {
        let (current, vault) = (nav.current.clone(), vault.clone());
        memo(move || (current.get(), vault.get().revision))
    };
    let shadow = shadow::popover();
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::MENU)
        Pickable::IGNORE
        Children [
            if $open {
                let (close, nav, vault, shadow) = (nav.links.clone(), nav.clone(), vault.clone(), shadow.clone());
                @{scrim(move || close.set(false))}
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
                        match $key {
                            (id, _) => { @{lists(id, &vault.get_untracked(), &nav)} },
                        }
                    ]
                ]
            }
        ]
    }
}

/// "Backlinks" and "Outgoing links", each with its notes or a note that there are none.
fn lists(id: NoteId, data: &VaultData, nav: &Nav) -> Box<dyn Scene> {
    let note = &data.notes[id];
    let group = |title: &str, ids: &[NoteId], none: &str| -> Box<dyn Scene> {
        let rows: Vec<Box<dyn Scene>> = if ids.is_empty() {
            vec![Box::new(empty(none))]
        } else {
            ids.iter()
                .map(|&target| {
                    let (nav, linked) = (nav.clone(), &data.notes[target]);
                    Box::new(bsn! {
                        @{item()}
                        on(move |_: On<Activate>| nav.open(target, None))
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
        let nav = Nav::new(0);
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy_radioactive::view::ViewPlugin,
        ));
        app.world_mut()
            .spawn_scene(links_popover(nav.clone(), vault))
            .unwrap();
        app.update();
        let panels = |app: &mut App| {
            app.world_mut()
                .query_filtered::<(), With<LinksPanel>>()
                .iter(app.world())
                .count()
        };
        assert_eq!(panels(&mut app), 0);
        nav.links.set(true);
        app.update();
        app.update();
        assert_eq!(panels(&mut app), 1);
    }
}
