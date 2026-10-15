//! The sidebar's search field. While it has focus, a popover under it suggests the tags
//! matching what is typed (all of them when it is empty, a leading `#` is optional); picking
//! one filters the note list by it.

use bevsidian_core::signal::memo;
use bevsidian_core::signal::signal;
use bevy::{input_focus::InputFocus, prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::view;

use bevsidian_core::{Vault, nav::Nav};
use bevsidian_ui::prelude::*;

pub fn search(nav: Nav, vault: Vault) -> impl Scene {
    let focused = signal(|| false);
    let within = FocusWithin(focused.clone());
    let query = Some(nav.query.clone());
    // Tags matching the query, with their note counts.
    let suggestions = {
        let (query, vault) = (nav.query.clone(), vault.clone());
        memo(move || {
            let typed = query.get().trim().trim_start_matches('#').to_lowercase();
            vault
                .get()
                .tags
                .iter()
                .filter(|(name, _)| name.to_lowercase().contains(&typed))
                .map(|(name, notes)| (name.clone(), notes.len()))
                .collect::<Vec<_>>()
        })
    };
    let (rows, any) = (suggestions.clone(), suggestions);
    let (active, touch) = (nav.tag.clone(), nav.touch.clone());
    let shadow = shadow::popover();
    view! {
        Node { flex_direction: FlexDirection::Column, margin: UiRect::vertical(px(8)), flex_shrink: 0.0 }
        within
        Children [
            @InputField { @icon: "search", @placeholder: "Search notes…", @value: query }
            --
            if $focused && !$(any.with(Vec::is_empty)) {
                let (shadow, nav) = (shadow.clone(), nav.clone());
                Node {
                    position_type: PositionType::Absolute,
                    top: px(40),
                    left: px(0),
                    right: px(0),
                    flex_direction: FlexDirection::Column,
                    max_height: px(280),
                    padding: UiRect::all(px(4)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(radius::LG)),
                    overflow: Overflow::scroll_y(),
                }
                bevy::ui_widgets::ScrollArea
                BackgroundColor(color::CARD)
                BorderColor::all(color::BORDER)
                shadow
                GlobalZIndex(layer::MENU)
                Children [
                    @{group_label("Filter by tag")}
                    --
                    #[key(entry.clone())]
                    for entry in $rows {
                        let (name, notes) = entry;
                        let (nav, filter, text) = (nav.clone(), name.clone(), name.clone());
                        @{item()}
                        Node { height: $(if $touch { px(40) } else { px(32) }) }
                        Selected($($active.as_deref() == Some(name.as_str())))
                        on(move |_: On<Activate>, mut focus: ResMut<InputFocus>| {
                            nav.toggle_tag(&filter);
                            focus.clear();
                        })
                        Children [
                            @{hash()}
                            --
                            @{item_label(&text)}
                            --
                            @{count(notes)}
                        ]
                    }
                ]
            }
        ]
    }
}
