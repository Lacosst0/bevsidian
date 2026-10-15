//! A segmented control (shadcn/ui `ToggleGroup`, single choice) bound to a value.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::view;

use crate::{
    binding::Binding,
    components::{
        button::{Size, Variant, button},
        typography::tinted,
    },
    surface::{Selected, Surface, Tint},
    tokens::{color, radius},
};

/// One button per `(value, label)`, the one equal to `value` selected; a click sets `value`.
/// When `value` matches no option (say, a stored setting from an older version), none is
/// selected until one is chosen.
pub fn segmented<T>(options: Vec<(T, String)>, value: Binding<T>) -> impl Scene
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let segments: Vec<Box<dyn Scene>> = options
        .into_iter()
        .enumerate()
        .map(|(index, (option, text))| {
            let (current, choose) = (value.clone(), value.clone());
            let chosen = option.clone();
            let divider = px(if index == 0 { 0.0 } else { 1.0 });
            Box::new(view! {
                @{button(Variant::Ghost, Size::Sm)}
                Surface::SEGMENT
                Node {
                    min_width: px(40),
                    padding: UiRect::horizontal(px(10)),
                    border: UiRect::left(divider),
                    border_radius: BorderRadius::ZERO,
                }
                Selected($($current == option))
                on(move |_: On<Activate>, mut commands: Commands| choose.set(&mut commands, chosen.clone()))
                Children [ @{tinted(&text, 13.0, FontWeight::MEDIUM, Tint::SEGMENT)} ]
            }) as Box<dyn Scene>
        })
        .collect();
    bsn! {
        Node {
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::MD)),
            overflow: Overflow::clip(),
            flex_shrink: 0.0,
        }
        BorderColor::all(color::INPUT)
        Children [ {segments} ]
    }
}
