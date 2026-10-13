//! Design tokens: colours, radii, shadows, layers and type.
//!
//! Colours follow shadcn/ui's semantic naming (`background`, `card`, `secondary`,
//! `foreground`, `muted-foreground`, `primary`, `border`, …) with Bevsidian's Obsidian-like
//! dark palette.

use bevy::{
    prelude::*,
    text::{FontSource, FontWeight, LineHeight},
};

/// Semantic colours.
pub mod color {
    use bevy::prelude::Color;

    /// App background.
    pub const BACKGROUND: Color = Color::srgb(0.118, 0.118, 0.118);
    /// Raised surfaces: sidebars, dialogs, popovers, cards.
    pub const CARD: Color = Color::srgb(0.149, 0.149, 0.149);
    /// Sunken wells, e.g. a live preview.
    pub const INSET: Color = Color::srgb(0.09, 0.09, 0.1);
    /// Secondary buttons, hover backgrounds, tracks.
    pub const SECONDARY: Color = Color::srgb(0.2, 0.2, 0.21);
    /// Hover on a secondary surface.
    pub const SECONDARY_HOVER: Color = Color::srgb(0.25, 0.25, 0.26);
    /// The selected item of a list or group.
    pub const SELECTED: Color = Color::srgb(0.24, 0.22, 0.32);
    /// Hover on a selected or primary surface.
    pub const SELECTED_HOVER: Color = Color::srgb(0.29, 0.27, 0.39);
    /// Floating notices (toasts).
    pub const ELEVATED: Color = Color::srgb(0.26, 0.26, 0.28);
    /// Dividers and container borders.
    pub const BORDER: Color = Color::srgb(0.21, 0.21, 0.21);
    /// Borders of outline buttons and inputs.
    pub const INPUT: Color = Color::srgb(0.27, 0.27, 0.29);
    /// Body text.
    pub const FOREGROUND: Color = Color::srgb(0.855, 0.855, 0.855);
    /// Secondary text.
    pub const MUTED_FOREGROUND: Color = Color::srgb(0.6, 0.6, 0.6);
    /// Hints, captions, section labels, placeholders.
    pub const SUBTLE_FOREGROUND: Color = Color::srgb(0.42, 0.42, 0.42);
    /// Brand accent: links, active icons, switches.
    pub const PRIMARY: Color = Color::srgb(0.659, 0.545, 0.98);
    /// Tinted primary background (badges, icon tiles).
    pub const PRIMARY_SOFT: Color = Color::srgba(0.545, 0.424, 0.937, 0.18);
    /// Stronger tint: hovered soft badges.
    pub const PRIMARY_SOFT_HOVER: Color = Color::srgba(0.545, 0.424, 0.937, 0.3);
    /// Selected soft badges.
    pub const PRIMARY_STRONG: Color = Color::srgba(0.659, 0.545, 0.98, 0.45);
    /// Behind dialogs and drawers.
    pub const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
    /// Drop shadows.
    pub const SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);
}

/// Corner radii (logical px).
pub mod radius {
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
    pub const XL: f32 = 12.0;
    pub const FULL: f32 = 999.0;
}

/// Stacking order of overlays (`GlobalZIndex`).
pub mod layer {
    pub const DRAWER_BACKDROP: i32 = 40;
    pub const DRAWER: i32 = 50;
    pub const TOOLBAR: i32 = 90;
    pub const COMMAND: i32 = 100;
    pub const DIALOG: i32 = 110;
    pub const KEYBOARD: i32 = 120;
    pub const MENU: i32 = 150;
    pub const TOAST: i32 = 160;
    pub const STATS: i32 = 200;
}

/// Font sizes (logical px).
pub mod size {
    /// Captions, section labels, badges.
    pub const XS: f32 = 11.5;
    /// Buttons, secondary text.
    pub const SM: f32 = 12.5;
    /// List items and body UI text.
    pub const BASE: f32 = 14.0;
    /// Large buttons and items.
    pub const LG: f32 = 15.0;
    /// Dialog titles.
    pub const TITLE: f32 = 16.0;
}

/// A soft drop shadow for floating toolbars and menus.
pub fn shadow_md() -> BoxShadow {
    BoxShadow::new(
        color::SHADOW,
        Val::Px(0.0),
        Val::Px(6.0),
        Val::Px(0.0),
        Val::Px(20.0),
    )
}

/// A larger shadow for dialogs and drawers.
pub fn shadow_lg() -> BoxShadow {
    BoxShadow::new(
        color::SHADOW,
        Val::Px(0.0),
        Val::Px(8.0),
        Val::Px(0.0),
        Val::Px(24.0),
    )
}

/// The UI font (Inter).
pub fn font(size: f32, weight: FontWeight) -> TextFont {
    TextFont {
        font: FontSource::families("Inter, sans-serif"),
        font_size: size.into(),
        weight,
        ..default()
    }
}

/// The monospace font (`JetBrains Mono`).
pub fn mono(size: f32) -> TextFont {
    TextFont {
        font: FontSource::families("JetBrains Mono, monospace"),
        font_size: size.into(),
        ..default()
    }
}

/// The UI font at `size` and `weight`, as a scene patch for a `Text`.
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
    let node = Node {
        width: Val::Px(size),
        height: Val::Px(size),
        flex_shrink: 0.0,
        ..default()
    };
    bsn! { @{insert(font)} @{insert(LineHeight::RelativeToFont(1.0))} @{insert(node)} }
}

/// Inserts a ready-made component value from a scene.
pub fn insert<C: Component + Clone>(value: C) -> impl Scene {
    bevy::ecs::template::template(move |_: &mut bevy::ecs::template::TemplateContext| {
        Ok(value.clone())
    })
}
