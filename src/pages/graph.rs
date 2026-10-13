//! The graph page: the rendered graph with its reading paths, legend and zoom controls.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{memo, view};

use bevsidian_core::{Nav, Vault};
use bevsidian_graph::{GraphViewport, legend, zoom_controls};
use bevsidian_ui::prelude::*;

pub fn graph_page(nav: Nav, vault: Vault) -> impl Scene {
    bsn! {
        Node { flex_grow: 1.0, flex_basis: px(0), overflow: Overflow::clip() }
        GraphViewport
        Children [ @{paths(nav, vault)} -- @{legend()} -- @{zoom_controls()} ]
    }
}

/// The vault's reading paths: choosing one shows only its notes, in order; "All notes"
/// shows everything again. Hidden when the vault has no paths.
fn paths(nav: Nav, vault: Vault) -> impl Scene {
    let list = memo(move || {
        let data = vault.get();
        data.paths
            .iter()
            .map(|path| {
                let note = &data.notes[path.note];
                (path.note, note.icon, note.name.clone(), path.steps.len())
            })
            .collect::<Vec<_>>()
    });
    let (rows, any) = (list.clone(), list);
    let (current, all) = (nav.path.clone(), nav.path.clone());
    let shadow = shadow::popover();
    view! {
        Node { position_type: PositionType::Absolute, left: px(16), top: px(16) }
        Pickable::IGNORE
        Children [
            if !$(any.with(Vec::is_empty)) {
                let (shadow, nav, all) = (shadow.clone(), nav.clone(), all.clone());
                Node {
                    flex_direction: FlexDirection::Column,
                    width: px(220),
                    padding: UiRect::all(px(4)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(radius::LG)),
                }
                BackgroundColor(color::CARD)
                BorderColor::all(color::INPUT)
                shadow
                Children [
                    @{group_label("Paths")}
                    --
                    @{item()}
                    Selected($current.is_none())
                    on(move |_: On<Activate>| all.set(None))
                    Children [ @{item_icon("orbit")} -- @{item_label("All notes")} ]
                    --
                    #[key(entry.0)]
                    for entry in $rows {
                        let (note, icon, name, steps) = entry;
                        let choose = nav.path.clone();
                        @{item()}
                        Selected($current == Some(note))
                        on(move |_: On<Activate>| choose.update(|path| {
                            *path = if *path == Some(note) { None } else { Some(note) };
                        }))
                        Children [ @{item_icon_char(icon)} -- @{item_label(&name)} -- @{count(steps)} ]
                    }
                ]
            }
        ]
    }
}
