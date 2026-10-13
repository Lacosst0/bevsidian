//! Inline alerts (shadcn/ui `Alert`): an icon, a title and a body. Notes render callouts the
//! same way.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::{icon::icon, typography::label},
    tokens::{color, font::size, radius},
};

/// Alert tone. Only warnings and errors get colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Default,
    Warning,
    Destructive,
}

impl Tone {
    fn colors(self) -> (Color, Color) {
        match self {
            Self::Default => (color::FOREGROUND, color::BORDER),
            Self::Warning => (color::WARNING, color::WARNING.with_alpha(0.1)),
            Self::Destructive => (color::DESTRUCTIVE, color::DESTRUCTIVE.with_alpha(0.1)),
        }
    }
}

/// An icon, a title and a muted body in a bordered box.
pub fn alert(tone: Tone, icon_name: &str, heading: &str, body: &str) -> impl Scene + use<> {
    let (accent, border) = tone.colors();
    bsn! {
        Node {
            column_gap: px(12),
            padding: UiRect::axes(px(16), px(14)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::XL)),
            min_width: px(0),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(border)
        Children [
            @{icon(icon_name, 16.0, accent)} Node { margin: UiRect::top(px(2)) }
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(4), flex_shrink: 1.0, min_width: px(0) }
            Children [
                @{label(heading, size::BASE, FontWeight::MEDIUM, accent)}
                --
                @{label(body, size::BASE, FontWeight::NORMAL, color::MUTED_FOREGROUND)}
            ]
        ]
    }
}
