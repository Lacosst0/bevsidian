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
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::{
    Vault,
    nav::{Nav, Studio},
};
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
#[component]
pub fn sidebar(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node {
            display: $(nav.with(|nav| {
                let shown = if nav.mode.drawer() { nav.drawer } else { !nav.sidebar_collapsed };
                if shown { Display::Flex } else { Display::None }
            })),
            position_type: $(if $nav.mode.drawer() { PositionType::Absolute } else { PositionType::Relative }),
            width: $(if $nav.mode.drawer() { Val::Percent(86.0) } else { px(264) }),
            max_width: px(320),
            height: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(8)),
            border: UiRect::right(px(1)),
        }
        GlobalZIndex($(if $nav.mode.drawer() { layer::DRAWER } else { 0 }))
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        BoxShadow($(drawer_shadow($nav.mode.drawer())))
        Children [
            @{header()}
            --
            @{search::search()}
            --
            @{tag_filter()}
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
                @{notes::notes()}
            ]
            --
            @{footer()}
        ]
    }
}

/// The drawer's shadow over the page (none inline).
fn drawer_shadow(drawer: bool) -> Vec<ShadowStyle> {
    let alpha = if drawer { 0.45 } else { 0.0 };
    BoxShadow::new(
        color::SHADOW.with_alpha(alpha),
        px(0),
        px(16),
        px(0),
        px(48),
    )
    .0
}

/// The app mark, the vault's note count, and on phones a close button.
#[component]
fn header(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>) -> impl Scene {
    let mark = icon_tile(
        glyph("book-open").chars().next().unwrap_or(' '),
        32.0,
        TileStyle::Inverted,
    );
    let count = insert(font(size::XS, FontWeight::NORMAL));
    view! {
        Node { height: px(48), align_items: AlignItems::Center, column_gap: px(10), padding: UiRect::horizontal(px(8)), flex_shrink: 0.0 }
        Children [
            // The mark, with the reader's sprite over it once drawn; a click opens "Make it
            // yours".
            Node { width: px(32), height: px(32), flex_shrink: 0.0 }
            Tooltip::new("Make it yours: draw your own sprite")
            on(|_: On<bevy::picking::events::PointerClick>, mut n: ResMut<Nav>| n.studio = Some(Studio::Sprite))
            Children [
                @{mark}
                --
                Node { position_type: PositionType::Absolute, left: px(4), top: px(4), width: px(24), height: px(24) }
                ImageNode::default()
                crate::overlays::studio::YourMark
                Visibility::Hidden
                Pickable::IGNORE
            ]
            --
            Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, min_width: px(0) }
            Children [
                @{label("Bevsidian", size::BASE, FontWeight::SEMIBOLD, color::FOREGROUND)}
                --
                Text($(match $vault.notes.len() {
                    1 => "1 note".to_owned(),
                    n => format!("{n} notes"),
                }))
                @{count}
                TextColor(color::MUTED_FOREGROUND)
                Pickable::IGNORE
            ]
            --
            if $nav.mode.drawer() {
                @{icon_button(Variant::Ghost, Size::IconLg, "x")}
                Node { margin: UiRect::right(px(-8)) }
                on(|_: On<Activate>, mut n: ResMut<Nav>| n.drawer = false)
            }
        ]
    }
}

/// The active tag filter, as a filled badge that clears it.
#[component]
fn tag_filter(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { flex_wrap: FlexWrap::Wrap, padding: UiRect::horizontal(px(4)), flex_shrink: 0.0 }
        Children [
            match $nav.tag {
                Some(name) => {
                    Node { column_gap: px(6), align_items: AlignItems::Center, margin: UiRect::bottom(px(8)) }
                    Children [
                        @{caption("Filtered by")}
                        --
                        @{badge_button(&format!("#{name}  ×"))}
                        Selected(true)
                        on(|_: On<Activate>, mut n: ResMut<Nav>| n.tag = None)
                    ]
                },
                None => {},
            }
        ]
    }
}

/// Settings, pinned under the scrolling part.
#[component]
fn footer(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { flex_direction: FlexDirection::Column, padding: UiRect::top(px(8)), border: UiRect::top(px(1)), flex_shrink: 0.0 }
        BorderColor::all(color::BORDER)
        Children [
            @{item()}
            Node { height: $(if $nav.touch { px(44) } else { px(32) }) }
            on(|_: On<Activate>, mut n: ResMut<Nav>| n.settings = !n.settings)
            Children [
                @{item_icon("sliders-horizontal")}
                --
                @{item_label("Settings")}
                --
                if !$nav.touch {
                    @{kbd_group("Ctrl ,")}
                }
            ]
        ]
    }
}

/// Dims the page behind the open drawer; tapping it closes the drawer.
#[component]
pub fn backdrop(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DRAWER_BACKDROP)
        Pickable::IGNORE
        Children [
            if $nav.drawer && $nav.mode.drawer() {
                @{scrim(|world| world.resource_mut::<Nav>().drawer = false)}
            }
        ]
    }
}
