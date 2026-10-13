//! Floating surfaces (shadcn/ui `Popover`, `DropdownMenu`, `ContextMenu`): the panel, its
//! items and separators.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::{
        item::{item, item_icon},
        typography::{label, tinted},
    },
    surface::Tint,
    tokens::{color, font::size, radius, shadow},
};

/// The floating panel.
pub fn popover() -> impl Scene {
    let shadow = shadow::popover();
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(4)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::LG)),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        shadow
    }
}

/// A menu row: icon, label and, on keyboards, its shortcut.
pub fn menu_item(icon: &str, text: &str, shortcut: Option<&str>) -> impl Scene + use<> {
    let shortcut: Vec<Box<dyn Scene>> = shortcut
        .map(|keys| {
            Box::new(label(
                keys,
                size::XS,
                FontWeight::NORMAL,
                color::MUTED_FOREGROUND,
            )) as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    bsn! {
        @{item()}
        Node { border_radius: BorderRadius::all(px(radius::SM)) }
        Children [
            @{item_icon(icon)}
            --
            @{tinted(text, size::BASE, FontWeight::NORMAL, Tint::TEXT)} Node { flex_grow: 1.0 }
            --
            {shortcut}
        ]
    }
}

/// A line between groups of menu items, edge to edge.
pub fn menu_separator() -> impl Scene {
    bsn! {
        Node { height: px(1), margin: UiRect::axes(px(-4), px(4)), flex_shrink: 0.0 }
        BackgroundColor(color::BORDER)
        Pickable::IGNORE
    }
}
