//! Badges (shadcn/ui `Badge`): small labelled pills, static or clickable.

use bevy::{
    input_focus::tab_navigation::TabIndex, picking::cursor::EntityCursor, prelude::*,
    text::FontWeight, window::SystemCursorIcon,
};

use crate::{
    components::typography::{label, tinted},
    surface::{Surface, SurfaceBorder, Tint},
    tokens::{color, font::size, radius},
};

/// Badge style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// White fill: an active filter.
    Default,
    /// Grey fill: counts.
    Secondary,
    /// Border only: tags.
    #[default]
    Outline,
}

/// A static badge showing `text`.
pub fn badge(variant: BadgeVariant, text: &str) -> impl Scene + use<> {
    let (background, border, foreground) = match variant {
        BadgeVariant::Default => (color::PRIMARY, color::PRIMARY, color::PRIMARY_FOREGROUND),
        BadgeVariant::Secondary => (color::ACCENT, color::ACCENT, color::MUTED_FOREGROUND),
        BadgeVariant::Outline => (Color::NONE, color::INPUT, color::BODY),
    };
    bsn! {
        Node {
            height: px(22),
            padding: UiRect::horizontal(px(8)),
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::MD)),
            flex_shrink: 0.0,
        }
        BackgroundColor(background)
        BorderColor::all(border)
        Pickable::IGNORE
        Children [ @{label(text, size::XS, FontWeight::MEDIUM, foreground)} ]
    }
}

/// A clickable outline badge (a tag); `Selected(...)` fills it, like a [`BadgeVariant::Default`].
pub fn badge_button(text: &str) -> impl Scene + use<> {
    bsn! {
        Node {
            height: px(24),
            padding: UiRect::horizontal(px(8)),
            column_gap: px(6),
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::MD)),
            flex_shrink: 0.0,
        }
        Surface::SEGMENT
        SurfaceBorder { idle: color::INPUT, selected: color::PRIMARY }
        BorderColor::all(color::INPUT)
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
        Children [ @{tinted(text, size::XS, FontWeight::MEDIUM, Tint::SEGMENT)} ]
    }
}
