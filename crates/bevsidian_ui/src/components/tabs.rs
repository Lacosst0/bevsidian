//! Tabs (shadcn/ui `Tabs` list): views of one thing on a grey track, e.g. Note | Graph.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::{
        button::{Size, Variant, button},
        typography::tinted,
    },
    surface::{Surface, SurfaceBorder, Tint},
    tokens::{color, radius},
};

/// The grey track holding [`tab`]s.
pub fn tab_list() -> impl Scene {
    bsn! {
        Node {
            height: px(32),
            padding: UiRect::all(px(3)),
            column_gap: px(2),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(radius::LG)),
            flex_shrink: 0.0,
        }
        BackgroundColor(color::ACCENT)
    }
}

/// One tab of a [`tab_list`]; set `Selected($(...))` on the active one.
pub fn tab(text: &str) -> impl Scene + use<> {
    bsn! {
        @{button(Variant::Ghost, Size::Xs)}
        Surface::TAB
        SurfaceBorder { idle: Color::NONE, selected: color::INPUT }
        Node { height: px(26), padding: UiRect::horizontal(px(10)), border: UiRect::all(px(1)) }
        Children [ @{tinted(text, 13.0, FontWeight::MEDIUM, Tint::MUTED)} ]
    }
}
