//! A labelled form row (shadcn/ui `Field`, horizontal): label and description, then the
//! control at the end.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::typography::{label, muted},
    tokens::{color, font::size},
};

/// `text` over an optional `description`, with `control` at the end of the row.
pub fn field(text: &str, description: Option<&str>, control: impl Scene) -> impl Scene {
    let description: Vec<Box<dyn Scene>> = description
        .map(|text| Box::new(muted(text)) as Box<dyn Scene>)
        .into_iter()
        .collect();
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(16), min_height: px(36) }
        Children [
            Node { flex_direction: FlexDirection::Column, row_gap: px(2), flex_grow: 1.0, flex_shrink: 1.0, min_width: px(0) }
            Children [
                @{label(text, size::BASE, FontWeight::MEDIUM, color::FOREGROUND)}
                --
                {description}
            ]
            --
            @{control}
        ]
    }
}
