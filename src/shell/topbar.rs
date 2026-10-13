//! The bar over the page.
//!
//! - Wide screens: sidebar toggle | back, forward, breadcrumb, links … Note / Graph tabs | edit.
//! - Phones (44 px touch targets): sidebar, back, the note's name, links … search, graph, edit.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::view;

use bevsidian_core::{
    Vault,
    nav::{LayoutMode, Nav, View},
};
use bevsidian_ui::prelude::*;

use crate::overlays::links::links_button;

pub fn topbar(nav: Nav, vault: Vault) -> impl Scene {
    let mode = nav.mode.clone();
    view! {
        Node {
            height: $(if $mode == LayoutMode::Compact { px(52) } else { px(48) }),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(8)),
            column_gap: px(2),
            border: UiRect::bottom(px(1)),
        }
        BorderColor::all(color::BORDER)
        Children [
            match $mode == LayoutMode::Compact {
                false => { @{wide(nav.clone(), vault.clone())} },
                true => { @{compact(nav.clone(), vault.clone())} },
            }
        ]
    }
}

fn wide(nav: Nav, vault: Vault) -> impl Scene {
    let (link_nav, link_vault) = (nav.clone(), vault.clone());
    let (back, forward, view) = (nav.back.clone(), nav.forward.clone(), nav.view.clone());
    let (sidebar, go_back, go_forward, note, graph) = (
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
    );
    view! {
        Node { flex_grow: 1.0, min_width: px(0), align_items: AlignItems::Center, column_gap: px(2) }
        Children [
            @{icon_button(Variant::Ghost, Size::IconSm, "panel-left")}
            Tooltip::new("Toggle sidebar")
            on(move |_: On<Activate>| sidebar.toggle_sidebar())
            --
            @{vertical_separator()}
            --
            @{icon_button(Variant::Ghost, Size::IconSm, "chevron-left")}
            Tooltip::with_keys("Back", "Alt ←")
            Disabled($back.is_empty())
            on(move |_: On<Activate>| go_back.go_back())
            --
            @{icon_button(Variant::Ghost, Size::IconSm, "chevron-right")}
            Tooltip::with_keys("Forward", "Alt →")
            Disabled($forward.is_empty())
            on(move |_: On<Activate>| go_forward.go_forward())
            --
            @{breadcrumb(nav.clone(), vault.clone())}
            --
            if $view == View::Note {
                let (nav, vault) = (link_nav.clone(), link_vault.clone());
                @{links_button(nav, vault, false)}
            }
            --
            Node { flex_grow: 1.0 }
            --
            @{tab_list()}
            Children [
                @{tab("Note")}
                Selected($view == View::Note)
                on(move |_: On<Activate>| note.show(View::Note))
                --
                @{tab("Graph")}
                Selected($view == View::Graph)
                on(move |_: On<Activate>| graph.show(View::Graph))
            ]
            --
            // The editor belongs to notes: hidden with its divider in the graph.
            Node { display: $(if $view == View::Note { Display::Flex } else { Display::None }), align_items: AlignItems::Center }
            Children [
                @{vertical_separator()}
                Node { margin: UiRect::new(px(8), px(4), px(0), px(0)) }
                --
                @{edit_button(nav.clone(), Size::IconSm)}
            ]
        ]
    }
}

fn compact(nav: Nav, vault: Vault) -> impl Scene {
    let (link_nav, link_vault) = (nav.clone(), vault.clone());
    let (back, view, current) = (nav.back.clone(), nav.view.clone(), nav.current.clone());
    let (sidebar, go_back, switcher, graph) = (nav.clone(), nav.clone(), nav.clone(), nav.clone());
    let title = insert(font(size::LG, FontWeight::MEDIUM));
    view! {
        Node { flex_grow: 1.0, min_width: px(0), align_items: AlignItems::Center }
        Children [
            @{icon_button(Variant::Ghost, Size::IconLg, "panel-left")}
            on(move |_: On<Activate>| sidebar.toggle_sidebar())
            --
            if !$back.is_empty() {
                let go_back = go_back.clone();
                @{icon_button(Variant::Ghost, Size::IconLg, "chevron-left")}
                on(move |_: On<Activate>| go_back.go_back())
            }
            --
            Node { flex_shrink: 1.0, min_width: px(0), overflow: Overflow::clip(), padding: UiRect::new(px(4), px(2), px(0), px(0)) }
            Children [
                Text($(match $view {
                    View::Note => vault.get().notes[$current].name.clone(),
                    View::Graph => "Graph".to_owned(),
                }))
                @{title}
                TextColor(color::FOREGROUND)
                TextLayout { linebreak: LineBreak::NoWrap }
            ]
            --
            if $view == View::Note {
                let (nav, vault) = (link_nav.clone(), link_vault.clone());
                @{links_button(nav, vault, true)}
            }
            --
            Node { flex_grow: 1.0 }
            --
            @{icon_button(Variant::Ghost, Size::IconLg, "search")}
            Tooltip::with_keys("Quick switcher", "Ctrl O")
            on(move |_: On<Activate>| switcher.open_switcher())
            --
            @{icon_button(Variant::Ghost, Size::IconLg, "waypoints")}
            Tooltip::with_keys("Graph", "Ctrl G")
            Selected($view == View::Graph)
            on(move |_: On<Activate>| graph.toggle_graph())
            --
            @{edit_button(nav.clone(), Size::IconLg)}
        ]
    }
}

/// The note's folders, muted, then its name; or "Graph". Cut at the end when too long.
fn breadcrumb(nav: Nav, vault: Vault) -> impl Scene {
    let (view, current) = (nav.view.clone(), nav.current.clone());
    let (folders, names) = (vault.clone(), vault);
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
            Text($(match $view {
                View::Note => {
                    let folder = folders.get().notes[$current].folder.clone();
                    if folder.is_empty() { String::new() } else { format!("{}  ›  ", folder.replace('/', "  ›  ")) }
                }
                View::Graph => String::new(),
            }))
            @{folder_font}
            TextColor(color::MUTED_FOREGROUND)
            TextLayout { linebreak: LineBreak::NoWrap }
            Node { flex_shrink: 0.0 }
            --
            Text($(match $view {
                View::Note => names.get().notes[$current].name.clone(),
                View::Graph => "Graph".to_owned(),
            }))
            @{name_font}
            TextColor(color::FOREGROUND)
            TextLayout { linebreak: LineBreak::NoWrap }
        ]
    }
}

/// Edits the note (`Ctrl+E`); while editing it shows a check mark that closes the editor.
fn edit_button(nav: Nav, size: Size) -> impl Scene {
    let (view, editing, toggle) = (nav.view.clone(), nav.editing.clone(), nav.clone());
    let (pen, done) = (glyph("pencil"), glyph("check"));
    view! {
        @{button(Variant::Ghost, size)}
        Tooltip::with_keys("Edit note", "Ctrl E")
        Node { display: $(if $view == View::Note { Display::Flex } else { Display::None }) }
        Selected($editing)
        on(move |_: On<Activate>| toggle.toggle_editing())
        Children [
            Text($(if $editing { done.clone() } else { pen.clone() }))
            @{icon_font(size.icon())}
            Tint::MUTED
            Pickable::IGNORE
        ]
    }
}
