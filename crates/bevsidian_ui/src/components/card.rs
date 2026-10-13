//! Containers (shadcn/ui `Card`).

use bevy::prelude::*;

use crate::tokens::{color, radius};

/// A bordered card-coloured panel.
pub fn card() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(16)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::XL)),
            min_width: px(0),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
    }
}

/// A page-coloured well for embedded content, such as a live preview.
pub fn inset() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(24)),
            min_width: px(0),
            min_height: px(120),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(color::BACKGROUND)
    }
}
