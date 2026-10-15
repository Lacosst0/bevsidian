//! `−  value  +` for a number with a range and a step.

use std::ops::RangeInclusive;

use bevsidian_core::signal::Signal;
use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::view;

use crate::{
    components::button::{Size, Variant, icon_button},
    surface::Disabled,
    tokens::{color, font::insert},
};

/// Steps `value` by `step` within `range`, shown through `format`. A button is disabled at
/// its end of the range, and a value outside the range (an old stored setting) snaps into it
/// on the first click.
pub fn stepper(
    value: Signal<u32>,
    range: RangeInclusive<u32>,
    step: u32,
    format: fn(u32) -> String,
) -> impl Scene {
    let (min, max) = (*range.start(), *range.end());
    let (less, more) = (value.clone(), value.clone());
    let font = insert(crate::font(13.0, FontWeight::NORMAL));
    view! {
        Node { align_items: AlignItems::Center, column_gap: px(4), flex_shrink: 0.0 }
        Children [
            @{icon_button(Variant::Outline, Size::IconSm, "minus")}
            Disabled($($value <= min))
            on(move |_: On<Activate>| less.update(|v| *v = v.saturating_sub(step).clamp(min, max)))
            --
            Node { width: px(56), justify_content: JustifyContent::Center }
            Pickable::IGNORE
            Children [
                Text($(format($value)))
                @{font}
                TextColor(color::MUTED_FOREGROUND)
                TextLayout { linebreak: LineBreak::NoWrap }
                Pickable::IGNORE
            ]
            --
            @{icon_button(Variant::Outline, Size::IconSm, "plus")}
            Disabled($($value >= max))
            on(move |_: On<Activate>| more.update(|v| *v = v.saturating_add(step).clamp(min, max)))
        ]
    }
}
