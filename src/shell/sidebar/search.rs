//! The sidebar's search field. While it has focus, a popover under it suggests the tags
//! matching what is typed (all of them when it is empty, a leading `#` is optional); picking
//! one filters the note list by it.

use bevy::{input_focus::InputFocus, prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::{Vault, VaultData, nav::Nav};
use bevsidian_ui::prelude::*;

/// Tags matching the query, with their note counts.
fn suggestions(nav: &Nav, vault: &VaultData) -> Vec<(String, usize)> {
    let typed = nav.query.trim().trim_start_matches('#').to_lowercase();
    vault
        .tags
        .iter()
        .filter(|(name, _)| name.to_lowercase().contains(&typed))
        .map(|(name, notes)| (name.clone(), notes.len()))
        .collect()
}

#[component]
pub fn search(
    nav: Watch<Res<Nav>>,
    vault: Watch<Res<Vault>>,
    focused: Watch<Local<bool>>,
) -> impl Scene {
    let within = FocusWithin(Binding::local(focused.clone()));
    let query = Some(Binding::resource(
        nav.clone(),
        |n: &Nav| n.query.clone(),
        |n, query| n.query = query,
    ));
    let shadow = shadow::popover();
    view! {
        Node { flex_direction: FlexDirection::Column, margin: UiRect::vertical(px(8)), flex_shrink: 0.0 }
        within
        Children [
            @InputField { @icon: "search", @placeholder: "Search notes…", @value: query }
            --
            if $focused && !$(suggestions(&$nav, &$vault).is_empty()) {
                let shadow = shadow.clone();
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
                    for entry in $(suggestions(&$nav, &$vault)) {
                        let (name, notes) = entry;
                        let (filter, text) = (name.clone(), name.clone());
                        @{item()}
                        Node { height: $(if $nav.touch { px(40) } else { px(32) }) }
                        Selected($nav.tag.as_deref() == Some(name.as_str()))
                        on(move |_: On<Activate>, mut n: ResMut<Nav>, mut focus: ResMut<InputFocus>| {
                            n.toggle_tag(&filter);
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
