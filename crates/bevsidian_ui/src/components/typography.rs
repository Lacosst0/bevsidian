//! Text: labels, titles and the small print around them.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    surface::Tint,
    tokens::{
        color,
        font::{insert, size},
    },
};

/// Non-interactive text in Geist.
pub fn label(text: &str, size: f32, weight: FontWeight, color: Color) -> impl Scene + use<> {
    let text = Text::new(text);
    let font = crate::font(size, weight);
    bsn! { text @{insert(font)} TextColor(color) Pickable::IGNORE }
}

/// A label that follows the state of the [`Surface`](crate::Surface) it sits in. It stays on
/// one line and is cut at the end when the row is too narrow.
pub fn tinted(text: &str, size: f32, weight: FontWeight, tint: Tint) -> impl Scene + use<> {
    bsn! {
        @{label(text, size, weight, tint.idle)}
        tint
        TextLayout { linebreak: LineBreak::NoWrap }
        Node { flex_shrink: 1.0, min_width: px(0), overflow: Overflow::clip() }
    }
}

/// A dialog or card title.
pub fn title(text: &str) -> impl Scene + use<> {
    label(text, size::TITLE, FontWeight::SEMIBOLD, color::FOREGROUND)
}

/// Secondary text: descriptions, hints.
pub fn muted(text: &str) -> impl Scene + use<> {
    label(text, size::SM, FontWeight::NORMAL, color::MUTED_FOREGROUND)
}

/// The smallest text: captions and metadata.
pub fn caption(text: &str) -> impl Scene + use<> {
    label(text, size::XS, FontWeight::NORMAL, color::MUTED_FOREGROUND)
}

/// The label over a group of rows ("Notes", "Tags"): sentence case, never shouted.
pub fn group_label(text: &str) -> impl Scene + use<> {
    bsn! {
        Node { height: px(32), align_items: AlignItems::Center, padding: UiRect::horizontal(px(8)), flex_shrink: 0.0 }
        Pickable::IGNORE
        Children [ @{label(text, size::XS, FontWeight::MEDIUM, color::MUTED_FOREGROUND)} ]
    }
}

/// What a list shows when it has nothing to show.
pub fn empty(text: &str) -> impl Scene + use<> {
    bsn! {
        Node { padding: UiRect::axes(px(8), px(6)) }
        Pickable::IGNORE
        Children [ @{muted(text)} ]
    }
}

/// A right-aligned number in a row (a tag's note count).
pub fn count(n: usize) -> impl Scene {
    bsn! {
        @{label(&n.to_string(), size::XS, FontWeight::NORMAL, color::MUTED_FOREGROUND)}
        Node { flex_shrink: 0.0 }
    }
}
