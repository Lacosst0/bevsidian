//! Colors and fonts shared by every view.

use bevy::{prelude::*, text::FontWeight};
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
