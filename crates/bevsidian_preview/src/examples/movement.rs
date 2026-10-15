//! `Transforms.md` / `Input.md`: a ship that flies to wherever you click. Its nose points up
//! (+y), as sprites are usually drawn, so it turns a quarter turn less than the direction.

use bevy::prelude::*;

use super::stage::{self, Sky, spawn_sky};
use crate::example::{Example, Setup, observer, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[
        startup!(spawn_sky),
        startup!(spawn_ship),
        observer!(set_target),
    ],
    functions: &[update!(fly), update!(turn)],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|_| {})
}

#[derive(Component)]
struct Ship;

/// Where the ship is going.
#[derive(Component)]
struct Target(Vec2);

fn spawn_ship(mut commands: Commands) {
    commands.spawn((
        Ship,
        Sprite::from_color(stage::colour(3), Vec2::new(12.0, 28.0)),
        Transform::from_xyz(-100.0, 0.0, 1.0),
    ));
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(6.0, 6.0)),
        Transform::from_xyz(0.0, 0.0, 0.5),
        Marker,
    ));
}

#[derive(Component)]
struct Marker;

fn set_target(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    ship: Query<Entity, With<Ship>>,
    mut marker: Query<&mut Transform, With<Marker>>,
    mut commands: Commands,
) {
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    for ship in &ship {
        commands.entity(ship).insert(Target(at.truncate()));
    }
    for mut marker in &mut marker {
        marker.translation = at.truncate().extend(0.5);
    }
}

fn fly(time: Res<Time>, mut ships: Query<(&mut Transform, &Target)>) {
    const SPEED: f32 = 90.0;
    for (mut transform, target) in &mut ships {
        let to_target = target.0 - transform.translation.truncate();
        // Never overshoot: move at most the distance that is left.
        let step = (SPEED * time.delta_secs()).min(to_target.length());
        transform.translation += (to_target.normalize_or_zero() * step).extend(0.0);
    }
}

fn turn(mut ships: Query<(&mut Transform, &Target)>) {
    for (mut transform, target) in &mut ships {
        let to_target = target.0 - transform.translation.truncate();
        if to_target.length() > 1.0 {
            let angle = to_target.to_angle() - std::f32::consts::FRAC_PI_2;
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}
