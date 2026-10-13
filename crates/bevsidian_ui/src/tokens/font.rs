//! Type: Geist for text, Geist Mono for code, Lucide for icons.

use bevy::{
    prelude::*,
    text::{FontSource, FontWeight, LineHeight},
};

/// Font sizes (logical px).
pub mod size {
    /// Group labels, counts, badges, keys.
    pub const XS: f32 = 12.0;
    /// Small buttons, sub-rows, descriptions.
    pub const SM: f32 = 13.0;
    /// Rows, buttons, body UI text.
    pub const BASE: f32 = 14.0;
    /// Phone rows and buttons.
    pub const LG: f32 = 15.0;
    /// Dialog titles.
    pub const TITLE: f32 = 18.0;
}

/// Geist at `size` and `weight`.
pub fn font(size: f32, weight: FontWeight) -> TextFont {
    TextFont {
        font: FontSource::families("Geist, sans-serif"),
        font_size: size.into(),
        weight,
        ..default()
    }
}

/// Geist Mono at `size`.
pub fn mono(size: f32) -> TextFont {
    TextFont {
        font: FontSource::families("Geist Mono, monospace"),
        font_size: size.into(),
        ..default()
    }
}

/// [`font`] as a scene patch for a `Text`.
pub fn ui_font(size: f32, weight: FontWeight) -> impl Scene {
    insert(font(size, weight))
}

/// The icon font at `size`; its text is an icon character (see [`bevsidian_core::icons`]).
///
/// The text node is exactly one em square with a one-em line: Lucide draws every icon centred
/// in its em box, so icons line up as squares (a default text box is 1.2 em tall).
pub fn icon_font(size: f32) -> impl Scene {
    let font = TextFont {
        font: FontSource::families("lucide"),
        font_size: size.into(),
        ..default()
    };
    // A patch, not a whole `Node`, so callers can still patch margins and the like.
    bsn! {
        @{insert(font)}
        @{insert(LineHeight::RelativeToFont(1.0))}
        Node { width: px(size), height: px(size), flex_shrink: 0.0 }
    }
}

/// Inserts a ready-made component value from a scene (for values that are not literals).
pub fn insert<C: Component + Clone>(value: C) -> impl Scene {
    bevy::ecs::template::template(move |_: &mut bevy::ecs::template::TemplateContext| {
        Ok(value.clone())
    })
}
