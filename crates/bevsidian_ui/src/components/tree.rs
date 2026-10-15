//! Hierarchy (shadcn/ui `SidebarMenuSub` + `Collapsible`): a disclosure chevron and the
//! indented list it opens, hung from a vertical guide line.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::view;

use crate::{
    binding::Binding,
    components::{
        button::{Size, Variant, button},
        icon::glyph,
    },
    surface::Tint,
    tokens::{color, font::icon_font},
};

/// A small chevron button that opens and closes a branch: right when closed, down when open.
/// It sits inside a row; clicking it does not activate the row.
pub fn disclosure(open: Binding<bool>) -> impl Scene {
    let (closed_glyph, open_glyph) = (glyph("chevron-right"), glyph("chevron-down"));
    let toggle = open.clone();
    view! {
        @{button(Variant::Ghost, Size::IconXs)}
        Node { width: px(22), height: px(22), margin: UiRect::right(px(-4)) }
        on(move |_: On<Activate>, mut commands: Commands| toggle.update(&mut commands, |was| !was))
        Children [
            Text($(if $open { open_glyph.clone() } else { closed_glyph.clone() }))
            @{icon_font(14.0)}
            TextColor(color::SUBTLE_FOREGROUND)
            Tint { idle: color::SUBTLE_FOREGROUND, hover: color::FOREGROUND, selected: color::FOREGROUND }
            Pickable::IGNORE
        ]
    }
}

/// The indented children of a branch, hung from a guide line under the parent's icon.
pub fn tree_children() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(1),
            margin: UiRect::new(px(15), px(0), px(2), px(4)),
            padding: UiRect::left(px(8)),
            border: UiRect::left(px(1)),
            min_width: px(0),
        }
        BorderColor::all(color::BORDER)
    }
}
