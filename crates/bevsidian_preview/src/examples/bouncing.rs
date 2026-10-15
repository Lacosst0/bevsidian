//! `Physics.md`: balls with gravity that bounce, and a click that drops another.

use bevy::{
    audio::{AudioSource, Volume},
    prelude::*,
};

use super::stage::{self, SIZE, Sky, colour, spawn_sky};
use crate::{
    example::{Example, Setup, observer, startup, update},
    sound::Slot,
    yours::Yours,
};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[
        startup!(spawn_balls),
        update!(gravity),
        update!(bounce),
        observer!(drop_ball),
    ],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(app: &World) -> Setup {
    // The reader's bounce sound ("Make it yours").
    let bounce = app
        .get_resource::<Yours>()
        .map_or_else(Handle::default, |yours| yours.sound(Slot::Bounce));
    Box::new(move |world| {
        world.insert_resource(Sounds {
            bounce: bounce.clone(),
        });
    })
}

#[derive(Resource)]
struct Sounds {
    bounce: Handle<AudioSource>,
}

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ball {
    radius: f32,
}

const GRAVITY: f32 = -400.0;
const FLOOR: f32 = -SIZE.y / 2.0;

fn ball(radius: f32, at: Vec2, velocity: Vec2) -> impl Bundle {
    (
        Ball { radius },
        Velocity(velocity),
        Sprite::from_color(colour(3), Vec2::splat(radius * 2.0)),
        Transform::from_translation(at.extend(1.0)),
    )
}

fn spawn_balls(mut commands: Commands) {
    commands.spawn(ball(10.0, Vec2::new(-60.0, 40.0), Vec2::new(30.0, 0.0)));
    commands.spawn(ball(14.0, Vec2::new(50.0, 60.0), Vec2::new(-20.0, 0.0)));
}

fn gravity(time: Res<Time>, mut balls: Query<(&mut Transform, &mut Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut balls {
        velocity.0.y += GRAVITY * dt;
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

fn bounce(
    mut balls: Query<(&mut Transform, &mut Velocity, &Ball)>,
    sounds: Res<Sounds>,
    mut commands: Commands,
) {
    for (mut transform, mut velocity, ball) in &mut balls {
        // Floor: flip the speed and lose a little energy.
        if transform.translation.y - ball.radius < FLOOR && velocity.0.y < 0.0 {
            transform.translation.y = FLOOR + ball.radius;
            // The harder it lands, the louder; the last little hops are silent.
            let speed = -velocity.0.y;
            if speed > 60.0 {
                let loudness = Volume::Linear((speed / 400.0).min(1.0));
                commands.spawn((
                    AudioPlayer::new(sounds.bounce.clone()),
                    PlaybackSettings::DESPAWN.with_volume(loudness),
                ));
            }
            velocity.0.y = -velocity.0.y * 0.8;
        }
        // Walls: flip sideways.
        let wall = SIZE.x / 2.0 - ball.radius;
        if transform.translation.x.abs() > wall {
            transform.translation.x = wall * transform.translation.x.signum();
            velocity.0.x = -velocity.0.x;
        }
    }
}

fn drop_ball(click: On<PointerClick>, sky: Query<(), With<Sky>>, mut commands: Commands) {
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    commands.spawn(ball(8.0, at.truncate(), Vec2::ZERO));
}
