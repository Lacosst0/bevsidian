//! Rows (shadcn/ui `SidebarMenuButton`, `CommandItem`): an icon, a label and something
//! trailing, transparent until hovered, highlighted when [`Selected`](crate::Selected).

use bevy::{
    input_focus::tab_navigation::TabIndex, picking::cursor::EntityCursor, prelude::*,
    text::FontWeight, window::SystemCursorIcon,
};

use crate::{
    components::{
        icon::{glyph, icon_char},
        typography::tinted,
    },
    surface::{Surface, Tint},
    tokens::{color, font::size, radius},
};

/// An empty row; fill it with [`item_icon`], [`item_label`] and a trailing
/// [`count`](crate::count) or [`kbd_group`](crate::kbd_group). Patch `Node { height }` for
/// touch screens.
pub fn item() -> impl Scene {
    bsn! {
        Node {
            height: px(32),
            padding: UiRect::horizontal(px(8)),
            column_gap: px(8),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(radius::MD)),
            flex_shrink: 0.0,
            min_width: px(0),
        }
        Surface::GHOST
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
    }
}

/// The label of an [`item`]: takes the free space and is cut at the end when too long.
pub fn item_label(text: &str) -> impl Scene + use<> {
    bsn! { @{tinted(text, size::BASE, FontWeight::NORMAL, Tint::TEXT)} Node { flex_grow: 1.0 } }
}

/// A Lucide icon at the start of an [`item`], by name.
pub fn item_icon(name: &str) -> impl Scene + use<> {
    item_icon_char(glyph(name).chars().next().unwrap_or(' '))
}

/// A note icon (a character) at the start of an [`item`].
pub fn item_icon_char(icon: char) -> impl Scene {
    bsn! { @{icon_char(icon, 16.0, color::MUTED_FOREGROUND)} Tint::MUTED }
}

/// A row nested in a [`tree_children`](crate::tree_children) list: smaller, with a muted
/// label and an optional note icon.
pub fn sub_item(icon: Option<char>, text: &str) -> impl Scene + use<> {
    let icon: Vec<Box<dyn Scene>> = icon
        .map(|icon| {
            Box::new(bsn! { @{icon_char(icon, 14.0, color::MUTED_FOREGROUND)} Tint::MUTED })
                as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    bsn! {
        @{item()}
        Node { height: px(28) }
        Children [
            {icon}
            --
            @{tinted(text, size::SM, FontWeight::NORMAL, Tint::MUTED)} Node { flex_grow: 1.0 }
        ]
    }
}

/// The `#` before a tag name in a row.
pub fn hash() -> impl Scene {
    bsn! { Text("#") @{crate::ui_font(size::BASE, FontWeight::NORMAL)} TextColor(color::SUBTLE_FOREGROUND) Pickable::IGNORE }
}
