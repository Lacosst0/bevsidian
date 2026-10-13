//! Buttons (shadcn/ui `Button`): four variants, text or icon or both.

use bevy::{
    input_focus::tab_navigation::TabIndex, picking::cursor::EntityCursor, prelude::*,
    text::FontWeight, window::SystemCursorIcon,
};

use crate::{
    components::{icon::tinted_icon, typography::tinted},
    surface::{Surface, Tint},
    tokens::{color, radius},
};

/// Button style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// White: the one call to action of a dialog.
    Primary,
    /// Grey.
    Secondary,
    /// Bordered, with a faint fill.
    Outline,
    /// Transparent until hovered; muted icon that brightens.
    #[default]
    Ghost,
}

impl Variant {
    fn surface(self) -> Surface {
        match self {
            Self::Primary => Surface::PRIMARY,
            Self::Secondary => Surface::SECONDARY,
            Self::Outline => Surface::OUTLINE,
            Self::Ghost => Surface::GHOST,
        }
    }

    /// Colour of the label and icon.
    pub fn tint(self) -> Tint {
        match self {
            Self::Primary => Tint::fixed(color::PRIMARY_FOREGROUND),
            Self::Secondary | Self::Outline => Tint::fixed(color::FOREGROUND),
            Self::Ghost => Tint::MUTED,
        }
    }
}

/// Button size. `Icon*` sizes are square.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// 28 px: inside headers of code blocks and cards.
    Xs,
    /// 32 px.
    Sm,
    /// 36 px.
    #[default]
    Md,
    /// 40 px.
    Lg,
    /// 28 px square.
    IconXs,
    /// 32 px square: toolbars.
    IconSm,
    /// 36 px square.
    Icon,
    /// 44 px square: touch targets on phones.
    IconLg,
}

impl Size {
    /// Height, and width for icon sizes.
    pub fn height(self) -> f32 {
        match self {
            Self::Xs | Self::IconXs => 28.0,
            Self::Sm | Self::IconSm => 32.0,
            Self::Md | Self::Icon => 36.0,
            Self::Lg => 40.0,
            Self::IconLg => 44.0,
        }
    }

    /// Whether the button is a square holding one icon.
    pub fn is_icon(self) -> bool {
        matches!(
            self,
            Self::IconXs | Self::IconSm | Self::Icon | Self::IconLg
        )
    }

    /// Font size of the label.
    pub fn text(self) -> f32 {
        match self {
            Self::Xs | Self::IconXs => 12.0,
            Self::Sm | Self::IconSm => 13.0,
            _ => 14.0,
        }
    }

    /// Size of the icon.
    pub fn icon(self) -> f32 {
        match self {
            Self::Xs | Self::IconXs => 14.0,
            Self::IconLg => 18.0,
            _ => 16.0,
        }
    }

    fn padding(self) -> f32 {
        match self {
            Self::Xs => 8.0,
            Self::Sm => 12.0,
            Self::Md => 16.0,
            Self::Lg => 20.0,
            _ => 0.0,
        }
    }
}

/// A button with no content: add children (or use [`text_button`], [`icon_button`],
/// [`icon_text_button`]) and an `on(|_: On<Activate>| ...)` handler. Set `Selected(...)` to
/// show it pressed and `Disabled(...)` to turn it off.
pub fn button(variant: Variant, size: Size) -> impl Scene {
    let node = Node {
        width: if size.is_icon() {
            px(size.height())
        } else {
            Val::Auto
        },
        height: px(size.height()),
        padding: UiRect::horizontal(px(size.padding())),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: px(6),
        flex_shrink: 0.0,
        border: UiRect::all(px(if variant == Variant::Outline {
            1.0
        } else {
            0.0
        })),
        border_radius: BorderRadius::all(px(radius::MD)),
        ..default()
    };
    let surface = variant.surface();
    bsn! {
        node
        surface
        BorderColor::all(color::INPUT)
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
    }
}

/// A button with a text label.
pub fn text_button(variant: Variant, size: Size, text: &str) -> impl Scene + use<> {
    bsn! {
        @{button(variant, size)}
        Children [ @{tinted(text, size.text(), FontWeight::MEDIUM, variant.tint())} ]
    }
}

/// A square button showing one Lucide icon.
pub fn icon_button(variant: Variant, size: Size, name: &str) -> impl Scene + use<> {
    bsn! {
        @{button(variant, size)}
        Children [ @{tinted_icon(name, size.icon(), variant.tint())} ]
    }
}

/// A button with an icon before its label.
pub fn icon_text_button(
    variant: Variant,
    size: Size,
    name: &str,
    text: &str,
) -> impl Scene + use<> {
    bsn! {
        @{button(variant, size)}
        Children [
            @{tinted_icon(name, size.icon(), variant.tint())}
            --
            @{tinted(text, size.text(), FontWeight::MEDIUM, variant.tint())}
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_sizes_are_square_and_touch_size_is_44() {
        for size in [Size::IconXs, Size::IconSm, Size::Icon, Size::IconLg] {
            assert!(size.is_icon());
            assert_eq!(size.padding(), 0.0);
        }
        assert_eq!(Size::IconLg.height(), 44.0);
        assert!(!Size::Md.is_icon());
    }

    #[test]
    fn primary_text_is_dark_on_white() {
        assert_eq!(Variant::Primary.tint().idle, color::PRIMARY_FOREGROUND);
        assert_eq!(Variant::Primary.surface().idle, color::PRIMARY);
    }
}
