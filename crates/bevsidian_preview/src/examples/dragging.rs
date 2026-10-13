//! `Picking.md`: squares you can pick up and drop.

use bevy::prelude::*;

use super::stage::{self, colour, spawn_sky};
use crate::example::{Example, Setup, observer, startup};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[
        startup!(spawn_cards),
        observer!(lift),
        observer!(drag),
        observer!(drop),
    ],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        world.init_resource::<Top>();
    })
}

/// The highest z handed out: whatever you pick up goes on top.
#[derive(Resource, Default)]
struct Top(f32);

#[derive(Component)]
struct Card;

fn spawn_cards(mut commands: Commands) {
    for i in 0..3 {
        commands.spawn((
            Card,
            Sprite::from_color(colour(i * 2), Vec2::new(50.0, 70.0)),
            Transform::from_xyz(-70.0 + 70.0 * i as f32, 0.0, i as f32),
        ));
    }
}

fn lift(
    start: On<PointerDragStart>,
    mut top: ResMut<Top>,
    mut cards: Query<&mut Transform, With<Card>>,
) {
    if let Ok(mut transform) = cards.get_mut(start.entity) {
        top.0 += 1.0;
        transform.translation.z = 10.0 + top.0;
        transform.scale = Vec3::splat(1.1);
    }
}

fn drag(drag: On<PointerDrag>, mut cards: Query<&mut Transform, With<Card>>) {
    if let Ok(mut transform) = cards.get_mut(drag.entity) {
        // Screen y points down, world y up.
        transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.0);
    }
}

fn drop(end: On<PointerDragEnd>, mut cards: Query<&mut Transform, With<Card>>) {
    if let Ok(mut transform) = cards.get_mut(end.entity) {
        transform.scale = Vec3::ONE;
    }
}
