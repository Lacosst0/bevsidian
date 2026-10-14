//! A checkbox (shadcn/ui `Checkbox`) showing a boolean signal. Like shadcn's checkbox inside a
//! `Label`, it does not take clicks itself: put it in a row (an [`item`](crate::item)) that
//! toggles the value, so the whole row is the target.

use bevy::prelude::*;
use bevy_radioactive::view::{Signal, view};

use crate::{
    components::icon::glyph,
    tokens::{color, font::icon_font, radius},
};

/// Checked: a white box with a dark tick. Unchecked: an empty box with the input border.
pub fn checkbox(value: Signal<bool>) -> impl Scene {
    let tick = value.clone();
    view! {
        Node {
            width: px(16),
            height: px(16),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::SM)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
        }
        BackgroundColor($(if $value { color::PRIMARY } else { color::INPUT_BG }))
        BorderColor::all($(if $value { color::PRIMARY } else { color::INPUT }))
        Pickable::IGNORE
        Children [
            Text($(if $tick { glyph("check") } else { String::new() }))
            @{icon_font(12.0)}
            TextColor(color::PRIMARY_FOREGROUND)
            Pickable::IGNORE
        ]
    }
}
