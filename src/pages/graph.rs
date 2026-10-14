//! The graph page: the rendered graph with its reading paths, legend and zoom controls.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{memo, signal, view};

use bevsidian_core::{Nav, Vault};
use bevsidian_graph::{GraphViewport, hover_hint, legend, zoom_controls};
use bevsidian_ui::prelude::*;

pub fn graph_page(nav: Nav, vault: Vault) -> impl Scene {
    bsn! {
        Node { flex_grow: 1.0, flex_basis: px(0), overflow: Overflow::clip() }
        GraphViewport
        Children [ @{paths(nav, vault)} -- @{legend()} -- @{hover_hint()} -- @{zoom_controls()} ]
    }
}

/// The vault's reading paths, in a card that folds down to its header (folded at first on
/// phones): choosing one shows only its notes, in order; "All notes" shows everything again.
/// Choosing a path folds the card out of the way; folded, its header names the path. Hidden when the vault has no paths.
fn paths(nav: Nav, vault: Vault) -> impl Scene {
    let names = {
        let vault = vault.clone();
        memo(move || {
            let data = vault.get();
            data.paths
                .iter()
                .map(|path| {
                    let note = &data.notes[path.note];
                    (path.note, note.icon, note.name.clone(), path.steps.len())
                })
                .collect::<Vec<_>>()
        })
    };
    let followed = {
        let (path, names) = (nav.path.clone(), names.clone());
        memo(move || {
            let path = path.get()?;
            names.with(|names| names.iter().find(|n| n.0 == path).map(|n| n.2.clone()))
        })
    };
    let open = signal({
        let mode = nav.mode.clone();
        move || !mode.get_untracked().drawer()
    });
    let (toggle, fold) = (open.clone(), open.clone());
    let (rows, any) = (names.clone(), names);
    let (current, all) = (nav.path.clone(), nav.path.clone());
    let shadow = shadow::popover();
    view! {
        Node { position_type: PositionType::Absolute, left: px(16), top: px(16) }
        Pickable::IGNORE
        Children [
            if !$(any.with(Vec::is_empty)) {
                let (shadow, nav, all, toggle, followed, fold) = (
                    shadow.clone(),
                    nav.clone(),
                    all.clone(),
                    toggle.clone(),
                    followed.clone(),
                    fold.clone(),
                );
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
                    @{item()}
                    Tooltip::new("Show or hide paths")
                    on(move |_: On<Activate>| toggle.update(|shown| *shown = !*shown))
                    Children [
                        @{item_icon("waypoints")}
                        --
                        @{item_label("Paths")}
                        Node { flex_grow: 0.0 }
                        --
                        // Folded, which path is followed.
                        Text($(if $open { String::new() } else { followed.get().unwrap_or_default() }))
                        @{insert(font(size::XS, bevy::text::FontWeight::NORMAL))}
                        TextColor(color::MUTED_FOREGROUND)
                        TextLayout { linebreak: LineBreak::NoWrap }
                        Node { flex_grow: 1.0, flex_shrink: 1.0, min_width: px(0), overflow: Overflow::clip() }
                        Pickable::IGNORE
                        --
                        Text($(glyph(if $open { "chevron-down" } else { "chevron-right" })))
                        @{icon_font(14.0)}
                        TextColor(color::MUTED_FOREGROUND)
                        Pickable::IGNORE
                    ]
                    --
                    if $open {
                        let (nav, all, fold) = (nav.clone(), all.clone(), fold.clone());
                        Node { flex_direction: FlexDirection::Column }
                        Children [
                            @{separator()}
                            Node { margin: UiRect::vertical(px(4)) }
                            --
                            @{item()}
                            Selected($current.is_none())
                            on(move |_: On<Activate>| all.set(None))
                            Children [ @{item_icon("orbit")} -- @{item_label("All notes")} ]
                            --
                            #[key(entry.0)]
                            for entry in $rows {
                                let (note, icon, name, steps) = entry;
                                let (choose, fold) = (nav.path.clone(), fold.clone());
                                @{item()}
                                Selected($current == Some(note))
                                on(move |_: On<Activate>| {
                                    choose.update(|path| {
                                        *path = if *path == Some(note) { None } else { Some(note) };
                                    });
                                    // Out of the way of the path's line; the header names it.
                                    fold.set(false);
                                })
                                Children [ @{item_icon_char(icon)} -- @{item_label(&name)} -- @{count(steps)} ]
                            }
                        ]
                    }
                ]
            }
        ]
    }
}
