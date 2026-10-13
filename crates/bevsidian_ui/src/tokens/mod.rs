//! Design tokens. Every colour, size and radius the components use comes from here.

pub mod color;
pub mod font;

/// Corner radii (logical px).
pub mod radius {
    /// Keys, inline code.
    pub const SM: f32 = 4.0;
    /// Buttons, inputs, rows.
    pub const MD: f32 = 6.0;
    /// Menus, tab tracks, tiles.
    pub const LG: f32 = 8.0;
    /// Alerts, cards, code blocks.
    pub const XL: f32 = 10.0;
    /// Dialogs, preview frames, note icons.
    pub const XXL: f32 = 12.0;
    /// Pills: switches, counts.
    pub const FULL: f32 = 999.0;
}

/// Spacing scale (logical px): 4, 8, 12, 16, 24, 32.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
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
    pub const TOOLTIP: i32 = 170;
    pub const STATS: i32 = 200;
}

/// Drop shadows.
pub mod shadow {
    use bevy::prelude::*;

    use super::color;

    /// Menus, toasts, floating toolbars.
    pub fn popover() -> BoxShadow {
        BoxShadow::new(color::SHADOW, px(0), px(10), px(0), px(30))
    }

    /// Dialogs and drawers.
    pub fn dialog() -> BoxShadow {
        BoxShadow::new(color::SHADOW, px(0), px(16), px(0), px(48))
    }
}
