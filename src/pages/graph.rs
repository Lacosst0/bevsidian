//! The graph page: the map with its quest log, legend, edit bar and zoom controls.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{memo, signal, view};

use bevsidian_core::{Nav, Vault, vault::Quest};
use bevsidian_graph::{GraphViewport, edit_bar, hover_hint, legend, route_color, zoom_controls};
use bevsidian_ui::prelude::*;

pub fn graph_page(nav: Nav, vault: Vault) -> impl Scene {
    bsn! {
        Node { flex_grow: 1.0, flex_basis: px(0), overflow: Overflow::clip() }
        GraphViewport
        Children [
            @{quest_log(nav, vault)} -- @{legend()} -- @{hover_hint()} -- @{edit_bar()} -- @{zoom_controls()}
        ]
    }
}

/// The routes on the map, in a card that folds down to its header (folded at first on
/// phones): the main quest and the routes chosen in the checklist, each with how many of its
/// steps the reader has visited. Choosing one follows it (its line lights up, its steps are
/// numbered and each of its notes links to the next); each wears its colour on the map.
/// "Choose features" reopens the checklist and "Whole map" shows every note and line.
/// Hidden when the vault has no routes.
fn quest_log(nav: Nav, vault: Vault) -> impl Scene {
    let rows = {
        let (vault, quests) = (vault.clone(), nav.quests.clone());
        memo(move || {
            let data = vault.get();
            let chosen = quests.get().unwrap_or_default();
            data.paths
                .iter()
                .filter(|p| p.quest == Quest::Main || chosen.contains(&p.note))
                .map(|p| {
                    let note = &data.notes[p.note];
                    let tint = route_color(&data, p.note);
                    (p.note, note.icon, tint, note.name.clone(), p.steps.clone())
                })
                .collect::<Vec<_>>()
        })
    };
    let any = {
        let vault = vault.clone();
        memo(move || !vault.get().paths.is_empty())
    };
    let followed = {
        let (path, vault) = (nav.path.clone(), vault.clone());
        memo(move || {
            let path = path.get()?;
            Some(vault.get().notes[path].name.clone())
        })
    };
    let open = signal({
        let mode = nav.mode.clone();
        move || !mode.get_untracked().drawer()
    });
    let (toggle, fold) = (open.clone(), open.clone());
    let current = nav.path.clone();
    let whole = nav.whole_map.clone();
    let shadow = shadow::popover();
    view! {
        Node { position_type: PositionType::Absolute, left: px(16), top: px(16) }
        Pickable::IGNORE
        Children [
            if $any {
                let (shadow, nav, toggle, followed, fold) = (
                    shadow.clone(),
                    nav.clone(),
                    toggle.clone(),
                    followed.clone(),
                    fold.clone(),
                );
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
                    on(move |_: On<Activate>| toggle.update(|shown| *shown = !*shown))
                    Children [
                        @{item_icon("map")}
                        --
                        @{item_label("Your game")}
                        Node { flex_grow: 0.0 }
                        --
                        // Folded, which route is followed.
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
                        let (nav, fold) = (nav.clone(), fold.clone());
                        let (setup, all) = (nav.map_setup.clone(), nav.whole_map.clone());
                        Node { flex_direction: FlexDirection::Column }
                        Children [
                            @{separator()}
                            Node { margin: UiRect::vertical(px(4)) }
                            --
                            #[key(entry.0)]
                            for entry in $rows {
                                let (note, icon, tint, name, steps) = entry;
                                let (choose, fold, visited) = (nav.path.clone(), fold.clone(), nav.visited.clone());
                                let total = steps.len();
                                @{item()}
                                Selected($current == Some(note))
                                on(move |_: On<Activate>| {
                                    choose.update(|path| {
                                        *path = if *path == Some(note) { None } else { Some(note) };
                                    });
                                    // Out of the way of the route; the header names it.
                                    fold.set(false);
                                })
                                Children [
                                    // In the route's colour on the map.
                                    @{icon_char(icon, 16.0, tint)}
                                    --
                                    @{item_label(&name)}
                                    --
                                    Text($(visited.with(|v| format!("{}/{total}", steps.iter().filter(|s| v.contains(s)).count()))))
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
                            on(move |_: On<Activate>| setup.set(true))
                            Children [ @{item_icon("list-checks")} -- @{item_label("Choose features…")} ]
                            --
                            @{item()}
                            Selected($whole)
                            on(move |_: On<Activate>| all.update(|on| *on = !*on))
                            Children [ @{item_icon("orbit")} -- @{item_label("Whole map")} ]
                        ]
                    }
                ]
            }
        ]
    }
}
