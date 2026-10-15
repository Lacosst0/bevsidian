//! The graph page: the map with its quest log, legend, edit bar and zoom controls.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::{Nav, NoteId, Vault, VaultData, vault::Quest};
use bevsidian_graph::{GraphViewport, edit_bar, hover_hint, legend, route_color, zoom_controls};
use bevsidian_ui::prelude::*;

/// The map; `folded` starts its quest log folded (on phones).
pub fn graph_page(folded: bool) -> impl Scene {
    bsn! {
        Node { flex_grow: 1.0, flex_basis: px(0), overflow: Overflow::clip() }
        GraphViewport
        Children [
            @{quest_log(folded)} -- @{legend()} -- @{hover_hint()} -- @{edit_bar()} -- @{zoom_controls()}
        ]
    }
}

/// A route in the quest log: its note, icon, colour, name and steps.
type RouteRow = (NoteId, char, Color, String, Vec<NoteId>);

/// The main quest and the routes chosen in the checklist.
fn routes(nav: &Nav, vault: &VaultData) -> Vec<RouteRow> {
    let chosen = nav.quests.clone().unwrap_or_default();
    vault
        .paths
        .iter()
        .filter(|p| p.quest == Quest::Main || chosen.contains(&p.note))
        .map(|p| {
            let note = &vault.notes[p.note];
            let tint = route_color(vault, p.note);
            (p.note, note.icon, tint, note.name.clone(), p.steps.clone())
        })
        .collect()
}

/// The routes on the map, in a card that folds down to its header: the main quest and the
/// routes chosen in the checklist, each with how many of its steps the reader has visited.
/// Choosing one follows it (its line lights up, its steps are numbered and each of its notes
/// links to the next); each wears its colour on the map. "Choose features" reopens the
/// checklist and "Whole map" shows every note and line. Hidden when the vault has no routes.
#[component]
fn quest_log(
    nav: Watch<Res<Nav>>,
    vault: Watch<Res<Vault>>,
    folded: bool,
    #[init(!folded)] open: Watch<Local<bool>>,
) -> impl Scene {
    // Folds the log from a route's row (`open` can't be named in a handler that takes
    // system parameters).
    let fold = open.clone();
    let shadow = shadow::popover();
    view! {
        Node { position_type: PositionType::Absolute, left: px(16), top: px(16) }
        Pickable::IGNORE
        Children [
            if !$vault.paths.is_empty() {
                let (shadow, fold) = (shadow.clone(), fold.clone());
                Node {
                    flex_direction: FlexDirection::Column,
                    width: px(248),
                    padding: UiRect::all(px(4)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(radius::LG)),
                }
                BackgroundColor(color::CARD)
                BorderColor::all(color::INPUT)
                shadow
                Children [
                    @{item()}
                    Tooltip::new("Show or hide your routes")
                    on(move |_: On<Activate>| open.update(|shown| *shown = !*shown))
                    Children [
                        @{item_icon("map")}
                        --
                        @{item_label("Your game")}
                        Node { flex_grow: 0.0 }
                        --
                        // Folded, which route is followed.
                        Text($(if $open {
                            String::new()
                        } else {
                            $nav.path.map(|path| $vault.notes[path].name.clone()).unwrap_or_default()
                        }))
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
                        let fold = fold.clone();
                        Node { flex_direction: FlexDirection::Column }
                        Children [
                            @{separator()}
                            Node { margin: UiRect::vertical(px(4)) }
                            --
                            #[key(entry.0)]
                            for entry in $(routes(&$nav, &$vault)) {
                                let (note, icon, tint, name, steps) = entry;
                                let (total, fold) = (steps.len(), fold.clone());
                                @{item()}
                                Selected($nav.path == Some(note))
                                on(move |_: On<Activate>, mut n: ResMut<Nav>| {
                                    n.path = if n.path == Some(note) { None } else { Some(note) };
                                    // Out of the way of the route; the header names it.
                                    fold.set(false);
                                })
                                Children [
                                    // In the route's colour on the map.
                                    @{icon_char(icon, 16.0, tint)}
                                    --
                                    @{item_label(&name)}
                                    --
                                    Text($(nav.with(|nav| format!("{}/{total}", steps.iter().filter(|s| nav.visited.contains(s)).count()))))
                                    @{insert(font(size::XS, bevy::text::FontWeight::NORMAL))}
                                    TextColor(color::MUTED_FOREGROUND)
                                    Node { flex_shrink: 0.0 }
                                    Pickable::IGNORE
                                ]
                            }
                            --
                            @{separator()}
                            Node { margin: UiRect::vertical(px(4)) }
                            --
                            @{item()}
                            on(|_: On<Activate>, mut n: ResMut<Nav>| n.map_setup = true)
                            Children [ @{item_icon("list-checks")} -- @{item_label("Choose features…")} ]
                            --
                            @{item()}
                            Selected($nav.whole_map)
                            on(|_: On<Activate>, mut n: ResMut<Nav>| n.whole_map = !n.whole_map)
                            Children [ @{item_icon("orbit")} -- @{item_label("Whole map")} ]
                        ]
                    }
                ]
            }
        ]
    }
}
