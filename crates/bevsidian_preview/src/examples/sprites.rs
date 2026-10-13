//! `Sprites.md`: coloured squares, the order they are drawn in, and clicks that add more.

use bevy::prelude::*;

use super::stage::{self, Sky, spawn_sky};
use crate::example::{Example, Setup, observer, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[
        startup!(spawn_squares),
        update!(breathe),
        observer!(place_square),
    ],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        world.init_resource::<Placed>();
    })
}

#[derive(Resource, Default)]
struct Placed(usize);

#[derive(Component)]
struct Breathing;

fn spawn_squares(mut commands: Commands) {
    // Later spawns are not drawn on top: z decides.
    commands.spawn((
        Sprite::from_color(stage::colour(0), Vec2::splat(60.0)),
        Transform::from_xyz(-30.0, 0.0, 1.0),
    ));
    commands.spawn((
        Breathing,
        Sprite::from_color(stage::colour(3), Vec2::splat(60.0)),
        Transform::from_xyz(0.0, 10.0, 0.0),
    ));
    commands.spawn((
        Sprite::from_color(stage::colour(1), Vec2::new(90.0, 20.0)),
        Transform::from_xyz(40.0, -30.0, 2.0),
    ));
}

fn breathe(time: Res<Time>, mut squares: Query<&mut Transform, With<Breathing>>) {
    for mut transform in &mut squares {
        let scale = 1.0 + 0.15 * (time.elapsed_secs() * 3.0).sin();
        transform.scale = Vec3::splat(scale);
    }
}

fn place_square(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    mut placed: ResMut<Placed>,
    mut commands: Commands,
) {
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    placed.0 += 1;
    commands.spawn((
        Sprite::from_color(stage::colour(placed.0 + 2), Vec2::splat(24.0)),
        Transform::from_translation(at.truncate().extend(3.0 + placed.0 as f32 * 0.01)),
    ));
}
