//! Colors and fonts shared by every view.

use bevy::{
    prelude::*,
    text::{FontWeight, LineHeight},
};
use bevy_ui_commonmark::insert;

/// The dark palette (Obsidian-like).
pub mod palette {
    use bevy::prelude::Color;

    pub const BG: Color = Color::srgb(0.118, 0.118, 0.118);
    pub const BG_SIDEBAR: Color = Color::srgb(0.149, 0.149, 0.149);
    pub const BG_HOVER: Color = Color::srgb(0.2, 0.2, 0.21);
    pub const BG_ACTIVE: Color = Color::srgb(0.24, 0.22, 0.32);
    pub const BORDER: Color = Color::srgb(0.21, 0.21, 0.21);
    pub const TEXT: Color = Color::srgb(0.855, 0.855, 0.855);
    pub const MUTED: Color = Color::srgb(0.6, 0.6, 0.6);
    pub const FAINT: Color = Color::srgb(0.42, 0.42, 0.42);
    pub const ACCENT: Color = Color::srgb(0.659, 0.545, 0.98);
    pub const TAG_BG: Color = Color::srgba(0.545, 0.424, 0.937, 0.18);
}

/// The UI font (Inter) at `size` and `weight`.
pub fn ui_font(size: f32, weight: FontWeight) -> impl Scene {
    insert(TextFont {
        font: bevy::text::FontSource::families("Inter, sans-serif"),
        font_size: size.into(),
        weight,
        ..default()
    })
}

/// The icon font at `size`; its text is an icon character (see [`crate::icons`]).
///
/// The text node is exactly one em square with a one-em line: Lucide draws every icon centred
/// in its em box, so icons line up as squares (a default text box is 1.2 em tall).
pub fn icon_font(size: f32) -> impl Scene {
    let font = TextFont {
        font: bevy::text::FontSource::families("lucide"),
        font_size: size.into(),
        ..default()
    };
    let node = Node {
        width: Val::Px(size),
        height: Val::Px(size),
        flex_shrink: 0.0,
        ..default()
    };
    bsn! { @{insert(font)} @{insert(LineHeight::RelativeToFont(1.0))} @{insert(node)} }
}
