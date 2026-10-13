//! Transient notices (shadcn/ui `Sonner`).

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::{icon::icon, typography::label},
    tokens::{color, font::size, radius, shadow},
};

/// A short notice with a check mark, floating over the page.
pub fn toast(text: &str) -> impl Scene + use<> {
    let shadow = shadow::popover();
    bsn! {
        Node {
            column_gap: px(10),
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(16), px(12)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::XL)),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        shadow
        Pickable::IGNORE
        Children [
            @{icon("circle-check", 16.0, color::FOREGROUND)}
            --
            @{label(text, size::BASE, FontWeight::MEDIUM, color::FOREGROUND)}
        ]
    }
}
