//! A slider (shadcn/ui `Slider`) bound to a number: a thin track, the range up to the
//! value filled, and a round thumb. Bevy's headless [`Slider`] does the dragging (a click on
//! the track jumps there).

use std::ops::RangeInclusive;

use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::cursor::EntityCursor,
    prelude::*,
    ui_widgets::{
        Slider, SliderOrientation, SliderRange, SliderThumb, SliderValue, TrackClick, ValueChange,
    },
    window::SystemCursorIcon,
};
use bevy_radioactive::view::view;

use crate::{
    binding::Binding,
    tokens::{color, radius},
};

/// Thumb diameter and track height (shadcn: `size-4`, `h-1.5`).
const THUMB: f32 = 16.0;
const TRACK: f32 = 6.0;

/// The value a slider shows: [`sync`] keeps its [`SliderValue`] (which the headless slider
/// drags from, and which can only be replaced) equal to it.
#[derive(Component, Clone, Default)]
pub struct BoundSlider(Binding<f32>);

pub fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, sync);
}

fn sync(sliders: Query<(Entity, &BoundSlider, &SliderValue)>, mut commands: Commands) {
    for (entity, bound, value) in &sliders {
        let wanted = bound.0.get_untracked();
        if (wanted - value.0).abs() > f32::EPSILON {
            commands.entity(entity).insert(SliderValue(wanted));
        }
    }
}

/// `amount` within `range`; dragging the thumb or clicking the track sets it.
pub fn slider(amount: Binding<f32>, range: RangeInclusive<f32>) -> impl Scene {
    let (start, end) = (*range.start(), *range.end());
    // Not read with `$`, so the handler (which takes system parameters) may name it.
    let set = amount.clone();
    let share = move |v: f32| {
        let span = (end - start).max(f32::EPSILON);
        ((v - start) / span).clamp(0.0, 1.0) * 100.0
    };
    view! {
        Node {
            height: px(THUMB),
            flex_grow: 1.0,
            min_width: px(96),
            align_items: AlignItems::Center,
        }
        Slider { track_click: TrackClick::Snap, orientation: SliderOrientation::default() }
        // Set from `amount` by `sync` (a watch can't be read while building).
        SliderValue(start)
        BoundSlider(amount.clone())
        SliderRange::new(start, end)
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
        on(move |change: On<ValueChange<f32>>, mut commands: Commands| set.set(&mut commands, change.value))
        Children [
            // The track, and the range up to the value.
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                height: px(TRACK),
                border_radius: BorderRadius::all(px(radius::FULL)),
                overflow: Overflow::clip(),
            }
            BackgroundColor(color::ACCENT)
            Pickable::IGNORE
            Children [
                Node { height: Val::Percent(100.0), width: $(Val::Percent(share($amount))) }
                BackgroundColor(color::PRIMARY)
                Pickable::IGNORE
            ]
            --
            // The thumb travels the track less its own width, as the headless slider expects.
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(THUMB),
                top: px(0),
                bottom: px(0),
            }
            Pickable::IGNORE
            Children [
                Node {
                    position_type: PositionType::Absolute,
                    left: $(Val::Percent(share($amount))),
                    width: px(THUMB),
                    height: px(THUMB),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(radius::FULL)),
                }
                BackgroundColor(color::FOREGROUND)
                BorderColor::all(color::PRIMARY)
                SliderThumb
                Pickable::IGNORE
            ]
        ]
    }
}
