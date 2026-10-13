//! Dividers (shadcn/ui `Separator`).

use bevy::prelude::*;

use crate::tokens::color;

/// A horizontal line.
pub fn separator() -> impl Scene {
    bsn! {
        Node { height: px(1), width: Val::Percent(100.0), flex_shrink: 0.0 }
        BackgroundColor(color::BORDER)
        Pickable::IGNORE
    }
}

/// A short vertical line between groups of toolbar buttons.
pub fn vertical_separator() -> impl Scene {
    bsn! {
        Node { width: px(1), height: px(16), margin: UiRect::horizontal(px(4)), flex_shrink: 0.0 }
        BackgroundColor(color::BORDER)
        Pickable::IGNORE
    }
}
