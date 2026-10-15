//! Lucide icons, alone or on a tile.

use bevy::prelude::*;

use bevsidian_core::icons;

use crate::{
    surface::Tint,
    tokens::{color, font::icon_font, radius},
};

/// A Lucide icon's character by name (`"pencil"`), or the default note icon for an unknown
/// name, so a typo shows a file icon rather than nothing.
pub fn glyph(name: &str) -> String {
    icons::find(name).unwrap_or(icons::DEFAULT).to_string()
}

/// A Lucide icon by name.
pub fn icon(name: &str, size: f32, color: Color) -> impl Scene + use<> {
    let text = Text::new(glyph(name));
    bsn! { text @{icon_font(size)} TextColor(color) Pickable::IGNORE }
}

/// An icon character (a note's icon) rather than a name.
pub fn icon_char(icon: char, size: f32, color: Color) -> impl Scene {
    let text = Text::new(icon.to_string());
    bsn! { text @{icon_font(size)} TextColor(color) Pickable::IGNORE }
}

/// An icon that follows the state of the [`Surface`](crate::Surface) it sits in.
pub fn tinted_icon(name: &str, size: f32, tint: Tint) -> impl Scene + use<> {
    bsn! { @{icon(name, size, tint.idle)} tint }
}

/// How an icon is mounted: the shape says what kind of thing it is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileStyle {
    /// A grey square: something that exists (an entity, a scene, a note).
    #[default]
    Filled,
    /// An outlined square: something that runs (a system, an observer).
    Outlined,
    /// A white square: the brand mark.
    Inverted,
}

/// An icon centred on a rounded square `size` px wide.
pub fn icon_tile(icon: char, size: f32, style: TileStyle) -> impl Scene {
    let (background, border, foreground) = match style {
        TileStyle::Filled => (color::ACCENT, Color::NONE, color::FOREGROUND),
        TileStyle::Outlined => (Color::NONE, color::INPUT, color::FOREGROUND),
        TileStyle::Inverted => (color::PRIMARY, Color::NONE, color::PRIMARY_FOREGROUND),
    };
    let node = Node {
        width: px(size),
        height: px(size),
        flex_shrink: 0.0,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(px(if border == Color::NONE { 0.0 } else { 1.0 })),
        border_radius: BorderRadius::all(px(if size >= 40.0 {
            radius::XXL
        } else {
            radius::MD
        })),
        ..default()
    };
    // Lucide icons sit at a bit over half the tile.
    let glyph = (size * 0.55).round();
    bsn! {
        node
        BackgroundColor(background)
        BorderColor::all(border)
        Pickable::IGNORE
        Children [ @{icon_char(icon, glyph, foreground)} ]
    }
}

/// The big icon before a note's title: the note's icon, in `tint`, on a bordered
/// card-coloured square, sized for a title of `title_size` px and centred on its first line
/// (`line` px high).
pub fn note_icon(icon: char, tint: Color, title_size: f32, line: f32) -> impl Scene {
    let tile = (title_size * 1.35).round().max(32.0);
    let node = Node {
        width: px(tile),
        height: px(tile),
        flex_shrink: 0.0,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::top(px(((line - tile) / 2.0).round())),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(radius::XXL)),
        ..default()
    };
    bsn! {
        node
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        Pickable::IGNORE
        Children [ @{icon_char(icon, (tile * 0.5).round(), tint)} ]
    }
}
