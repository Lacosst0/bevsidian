//! Keyboard keys (shadcn/ui `Kbd`) and shortcut hints.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::typography::{caption, label},
    tokens::{color, radius},
};

/// One key: `"Ctrl"`, `"↵"`.
pub fn kbd(key: &str) -> impl Scene + use<> {
    bsn! {
        Node {
            min_width: px(20),
            height: px(20),
            padding: UiRect::horizontal(px(5)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(radius::SM)),
            flex_shrink: 0.0,
        }
        BackgroundColor(color::ACCENT)
        Pickable::IGNORE
        Children [ @{label(key, 11.5, FontWeight::MEDIUM, color::MUTED_FOREGROUND)} ]
    }
}

/// The keys of a shortcut, separated by spaces in `keys`: `"Ctrl O"`, `"↑ ↓"`.
pub fn kbd_group(keys: &str) -> impl Scene + use<> {
    let keys: Vec<Box<dyn Scene>> = keys
        .split_whitespace()
        .map(|key| Box::new(kbd(key)) as Box<dyn Scene>)
        .collect();
    bsn! {
        Node { column_gap: px(3), align_items: AlignItems::Center, flex_shrink: 0.0 }
        Pickable::IGNORE
        Children [ {keys} ]
    }
}

/// Shortcuts with what they do: `&[("Ctrl O", "switch"), ("Esc", "close")]`.
pub fn shortcuts(items: &[(&str, &str)]) -> impl Scene + use<> {
    let items: Vec<Box<dyn Scene>> = items
        .iter()
        .map(|(keys, action)| {
            Box::new(bsn! {
                Node { column_gap: px(6), align_items: AlignItems::Center }
                Children [ @{kbd_group(keys)} -- @{caption(action)} ]
            }) as Box<dyn Scene>
        })
        .collect();
    bsn! {
        Node { flex_wrap: FlexWrap::Wrap, column_gap: px(16), row_gap: px(6), align_items: AlignItems::Center }
        Pickable::IGNORE
        Children [ {items} ]
    }
}
