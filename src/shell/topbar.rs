//! The bar over the page.
//!
//! - Wide screens: sidebar toggle | back, forward, breadcrumb, links … Note / Graph tabs | edit.
//! - Phones (44 px touch targets): sidebar, back, the note's name, links … search, graph, edit.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::{
    Vault,
    nav::{LayoutMode, Nav, View},
};
use bevsidian_ui::prelude::*;

use crate::overlays::links::links_button;

#[component]
pub fn topbar(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node {
            height: $(if $nav.mode == LayoutMode::Compact { px(52) } else { px(48) }),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(8)),
            column_gap: px(2),
            border: UiRect::bottom(px(1)),
        }
        BorderColor::all(color::BORDER)
        Children [
            match $nav.mode == LayoutMode::Compact {
                false => { @{wide()} },
                true => { @{compact()} },
            }
        ]
    }
}

#[component]
fn wide(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { flex_grow: 1.0, min_width: px(0), align_items: AlignItems::Center, column_gap: px(2) }
        Children [
            @{icon_button(Variant::Ghost, Size::IconSm, "panel-left")}
            Tooltip::new("Toggle sidebar")
            on(|_: On<Activate>, mut n: ResMut<Nav>| n.toggle_sidebar())
            --
            @{vertical_separator()}
            --
            @{icon_button(Variant::Ghost, Size::IconSm, "chevron-left")}
            Tooltip::with_keys("Back", "Alt ←")
            Disabled($nav.back.is_empty())
            on(|_: On<Activate>, mut n: ResMut<Nav>| n.go_back())
            --
            @{icon_button(Variant::Ghost, Size::IconSm, "chevron-right")}
            Tooltip::with_keys("Forward", "Alt →")
            Disabled($nav.forward.is_empty())
            on(|_: On<Activate>, mut n: ResMut<Nav>| n.go_forward())
            --
            @{breadcrumb()}
            --
            if $nav.view == View::Note {
                @{links_button(false)}
            }
            --
            Node { flex_grow: 1.0 }
            --
            @{tab_list()}
            Children [
                @{tab("Note")}
                Selected($nav.view == View::Note)
                on(|_: On<Activate>, mut n: ResMut<Nav>| n.view = View::Note)
                --
                @{tab("Graph")}
                Selected($nav.view == View::Graph)
                on(|_: On<Activate>, mut n: ResMut<Nav>| n.view = View::Graph)
            ]
            --
            // The editor belongs to notes: hidden with its divider in the graph.
            Node { display: $(if $nav.view == View::Note { Display::Flex } else { Display::None }), align_items: AlignItems::Center }
            Children [
                @{vertical_separator()}
                Node { margin: UiRect::new(px(8), px(4), px(0), px(0)) }
                --
                @{edit_button(Size::IconSm)}
            ]
        ]
    }
}

#[component]
fn compact(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>) -> impl Scene {
    let title = insert(font(size::LG, FontWeight::MEDIUM));
    view! {
        Node { flex_grow: 1.0, min_width: px(0), align_items: AlignItems::Center }
        Children [
            @{icon_button(Variant::Ghost, Size::IconLg, "panel-left")}
            on(|_: On<Activate>, mut n: ResMut<Nav>| n.toggle_sidebar())
            --
            if !$nav.back.is_empty() {
                @{icon_button(Variant::Ghost, Size::IconLg, "chevron-left")}
                on(|_: On<Activate>, mut n: ResMut<Nav>| n.go_back())
            }
            --
            Node { flex_shrink: 1.0, min_width: px(0), overflow: Overflow::clip(), padding: UiRect::new(px(4), px(2), px(0), px(0)) }
            Children [
                Text($(match $nav.view {
                    View::Note => $vault.notes[$nav.current].name.clone(),
                    View::Graph => "Graph".to_owned(),
                }))
                @{title}
                TextColor(color::FOREGROUND)
                TextLayout { linebreak: LineBreak::NoWrap }
            ]
            --
            if $nav.view == View::Note {
                @{links_button(true)}
            }
            --
            Node { flex_grow: 1.0 }
            --
            @{icon_button(Variant::Ghost, Size::IconLg, "search")}
            Tooltip::with_keys("Quick switcher", "Ctrl O")
            on(|_: On<Activate>, mut n: ResMut<Nav>| n.open_switcher())
            --
            @{icon_button(Variant::Ghost, Size::IconLg, "waypoints")}
            Tooltip::with_keys("Graph", "Ctrl G")
            Selected($nav.view == View::Graph)
            on(|_: On<Activate>, mut n: ResMut<Nav>| n.toggle_graph())
            --
            @{edit_button(Size::IconLg)}
        ]
    }
}

/// The note's folders, muted, then its name; or "Graph". Cut at the end when too long.
#[component]
fn breadcrumb(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>) -> impl Scene {
    let (folder_font, name_font) = (
        insert(font(size::BASE, FontWeight::NORMAL)),
        insert(font(size::BASE, FontWeight::MEDIUM)),
    );
    view! {
        Node {
            flex_shrink: 1.0,
            min_width: px(0),
            overflow: Overflow::clip(),
            align_items: AlignItems::Center,
            margin: UiRect::horizontal(px(8)),
        }
        Children [
            Text($(match $nav.view {
                View::Note => {
                    let folder = $vault.notes[$nav.current].folder.clone();
                    if folder.is_empty() { String::new() } else { format!("{}  ›  ", folder.replace('/', "  ›  ")) }
                }
                View::Graph => String::new(),
            }))
            @{folder_font}
            TextColor(color::MUTED_FOREGROUND)
            TextLayout { linebreak: LineBreak::NoWrap }
            Node { flex_shrink: 0.0 }
            --
            Text($(match $nav.view {
                View::Note => $vault.notes[$nav.current].name.clone(),
                View::Graph => "Graph".to_owned(),
            }))
            @{name_font}
            TextColor(color::FOREGROUND)
            TextLayout { linebreak: LineBreak::NoWrap }
        ]
    }
}

/// Edits the note (`Ctrl+E`); while editing it shows a check mark that closes the editor.
#[component]
fn edit_button(nav: Watch<Res<Nav>>, size: Size) -> impl Scene {
    let (pen, done) = (glyph("pencil"), glyph("check"));
    view! {
        @{button(Variant::Ghost, size)}
        Tooltip::with_keys("Edit note", "Ctrl E")
        Node { display: $(if $nav.view == View::Note { Display::Flex } else { Display::None }) }
        Selected($nav.editing)
        on(|_: On<Activate>, mut n: ResMut<Nav>| n.toggle_editing())
        Children [
            Text($(if $nav.editing { done.clone() } else { pen.clone() }))
            @{icon_font(size.icon())}
            Tint::MUTED
            Pickable::IGNORE
        ]
    }
}
