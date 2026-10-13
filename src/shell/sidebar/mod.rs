//! The one sidebar: the vault, a search field (suggesting tags while it has focus), the note
//! tree with the open note's outline, and Settings at the bottom. Inline on wide screens (collapsible), a
//! drawer on phones.

mod notes;
mod search;

use bevy::{
    prelude::*,
    text::FontWeight,
    ui_widgets::{Activate, ScrollArea, ScrollIntoView},
};
use bevy_radioactive::view::view;

use bevsidian_core::{Vault, nav::Nav};
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        follow_selection.after(bevy::ui::UiSystems::Layout),
    );
}

/// A row the sidebar scrolls to when it becomes selected (the open note, the section being
/// read), so where you are is always in view.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct FollowSelection;

fn follow_selection(
    rows: Query<(Entity, &Selected, &ComputedNode), (With<FollowSelection>, Changed<Selected>)>,
    mut commands: Commands,
) {
    for (row, selected, node) in &rows {
        // Rows inside a folded branch have no size: nothing to show.
        if selected.0 && node.size.y > 0.0 {
            commands.trigger(ScrollIntoView { entity: row });
        }
    }
}

/// The sidebar column (or drawer).
pub fn sidebar(nav: Nav, vault: Vault) -> impl Scene {
    let (mode, drawer_open, collapsed) = (
        nav.mode.clone(),
        nav.drawer.clone(),
        nav.sidebar_collapsed.clone(),
    );
    view! {
        Node {
            display: $(if ($mode.drawer() && $drawer_open) || (!$mode.drawer() && !$collapsed) { Display::Flex } else { Display::None }),
            position_type: $(if $mode.drawer() { PositionType::Absolute } else { PositionType::Relative }),
            width: $(if $mode.drawer() { Val::Percent(86.0) } else { px(264) }),
            max_width: px(320),
            height: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(8)),
            border: UiRect::right(px(1)),
        }
        GlobalZIndex($(if $mode.drawer() { layer::DRAWER } else { 0 }))
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        BoxShadow::new(color::SHADOW.with_alpha($(if $mode.drawer() { 0.45 } else { 0.0 })), px(0), px(16), px(0), px(48))
        Children [
            @{header(nav.clone(), vault.clone())}
            --
            @{search::search(nav.clone(), vault.clone())}
            --
            @{tag_filter(nav.clone())}
            --
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_basis: px(0),
                min_height: px(0),
                min_width: px(0),
                overflow: Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Scroll },
            }
            ScrollArea
            Children [
                @{notes::notes(nav.clone(), vault.clone())}
            ]
            --
            @{footer(nav.clone())}
        ]
    }
}

/// The app mark, the vault's note count, and on phones a close button.
fn header(nav: Nav, vault: Vault) -> impl Scene {
    let mode = nav.mode.clone();
    let mark = icon_tile(
        glyph("book-open").chars().next().unwrap_or(' '),
        32.0,
        TileStyle::Inverted,
    );
    let count = insert(font(size::XS, FontWeight::NORMAL));
    view! {
        Node { height: px(48), align_items: AlignItems::Center, column_gap: px(10), padding: UiRect::horizontal(px(8)), flex_shrink: 0.0 }
        Children [
            @{mark}
            --
            Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, min_width: px(0) }
            Children [
                @{label("Bevsidian", size::BASE, FontWeight::SEMIBOLD, color::FOREGROUND)}
                --
                Text($(match vault.get().notes.len() {
                    1 => "1 note".to_owned(),
                    n => format!("{n} notes"),
                }))
                @{count}
                TextColor(color::MUTED_FOREGROUND)
                Pickable::IGNORE
            ]
            --
            if $mode.drawer() {
                let nav = nav.clone();
                @{icon_button(Variant::Ghost, Size::IconLg, "x")}
                Node { margin: UiRect::right(px(-8)) }
                on(move |_: On<Activate>| nav.close_drawer())
            }
        ]
    }
}

/// The active tag filter, as a filled badge that clears it.
fn tag_filter(nav: Nav) -> impl Scene {
    let tag = nav.tag.clone();
    view! {
        Node { flex_wrap: FlexWrap::Wrap, padding: UiRect::horizontal(px(4)), flex_shrink: 0.0 }
        Children [
            match $tag.clone() {
                Some(name) => {
                    let nav = nav.clone();
                    Node { column_gap: px(6), align_items: AlignItems::Center, margin: UiRect::bottom(px(8)) }
                    Children [
                        @{caption("Filtered by")}
                        --
                        @{badge_button(&format!("#{name}  ×"))}
                        Selected(true)
                        on(move |_: On<Activate>| nav.clear_tag())
                    ]
                },
                None => {},
            }
        ]
    }
}

/// Settings, pinned under the scrolling part.
fn footer(nav: Nav) -> impl Scene {
    let touch = nav.touch.clone();
    let open = nav.settings.clone();
    view! {
        Node { flex_direction: FlexDirection::Column, padding: UiRect::top(px(8)), border: UiRect::top(px(1)), flex_shrink: 0.0 }
        BorderColor::all(color::BORDER)
        Children [
            @{item()}
            Node { height: $(if $touch { px(44) } else { px(32) }) }
            on(move |_: On<Activate>| open.update(|shown| *shown = !*shown))
            Children [
                @{item_icon("sliders-horizontal")}
                --
                @{item_label("Settings")}
                --
                if !$touch {
                    @{kbd_group("Ctrl ,")}
                }
            ]
        ]
    }
}

/// Dims the page behind the open drawer; tapping it closes the drawer.
pub fn backdrop(nav: Nav) -> impl Scene {
    let (mode, drawer_open) = (nav.mode.clone(), nav.drawer.clone());
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DRAWER_BACKDROP)
        Pickable::IGNORE
        Children [
            if $drawer_open && $mode.drawer() {
                let nav = nav.clone();
                @{scrim(move || nav.close_drawer())}
            }
        ]
    }
}
