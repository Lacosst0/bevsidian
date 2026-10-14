//! The marker where the mouse wheel was pressed to autoscroll: a round badge with arrows
//! along the axes the page scrolls (as browsers draw it).

use bevy::prelude::*;
use bevy_ui_touchscroll::TouchScroll;

use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, show_marker);
}

/// Diameter of the marker (px).
const SIZE: f32 = 32.0;

#[derive(Component)]
struct AutoscrollMarker;

fn show_marker(
    scroll: Res<TouchScroll>,
    markers: Query<Entity, With<AutoscrollMarker>>,
    mut commands: Commands,
) {
    let running = scroll.autoscroll();
    // Already shown while running, or gone while not.
    if running.is_some() != markers.is_empty() {
        return;
    }
    for marker in &markers {
        commands.entity(marker).despawn();
    }
    let Some((origin, axes)) = running else {
        return;
    };
    let arrows = match (axes.x, axes.y) {
        (true, true) => "move",
        (true, false) => "chevrons-left-right",
        _ => "chevrons-up-down",
    };
    let (left, top) = (px(origin.x - SIZE / 2.0), px(origin.y - SIZE / 2.0));
    let raised = shadow::popover();
    commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: left,
                top: top,
                width: px(SIZE),
                height: px(SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::MAX,
            }
            BackgroundColor(color::CARD)
            BorderColor::all(color::BORDER)
            raised
            GlobalZIndex(layer::TOOLTIP)
            Pickable::IGNORE
            Children [ @{icon(arrows, 18.0, color::FOREGROUND)} ]
        })
        .insert(AutoscrollMarker);
}
