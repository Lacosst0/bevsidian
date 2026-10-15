//! `Game feel.md`: click the player to hit it. Each block adds one kind of feedback: squash,
//! blinking, sparks. Every hit plays the reader's hit sound.

use bevy::{audio::AudioSource, prelude::*};

use super::stage::{self, colour, spawn_sky};
use crate::{
    example::{Example, Setup, observer, startup, update},
    sound::Slot,
    yours::Yours,
};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[
        startup!(spawn_sky),
        startup!(spawn_player),
        observer!(on_hit),
    ],
    functions: &[
        update!(squash),
        update!(blink),
        observer!(burst),
        update!(sparks),
    ],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(app: &World) -> Setup {
    // The reader's hit sound ("Make it yours").
    let hit = app
        .get_resource::<Yours>()
        .map_or_else(Handle::default, |yours| yours.sound(Slot::Hit));
    Box::new(move |world| {
        world.insert_resource(Sounds { hit: hit.clone() });
    })
}

#[derive(Resource)]
struct Sounds {
    hit: Handle<AudioSource>,
}

#[derive(Component)]
struct Player;

/// 1.0 right after a hit, back to 0 over a moment.
#[derive(Component, Default)]
struct Squash(f32);

/// Blinking after a hit, as long as the timer runs.
#[derive(Component)]
struct Blink(Timer);

#[derive(Component)]
struct Spark {
    velocity: Vec2,
    life: Timer,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Squash::default(),
        Sprite::from_color(colour(3), Vec2::splat(40.0)),
        Transform::from_xyz(0.0, -20.0, 1.0),
    ));
    // The floor it stands on.
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.22, 0.3), Vec2::new(320.0, 20.0)),
        Transform::from_xyz(0.0, -50.0, 0.5),
    ));
}

fn on_hit(
    click: On<PointerClick>,
    mut player: Query<&mut Squash, With<Player>>,
    sounds: Res<Sounds>,
    mut commands: Commands,
) {
    let Ok(mut squash) = player.get_mut(click.entity) else {
        return;
    };
    squash.0 = 1.0;
    commands
        .entity(click.entity)
        .insert(Blink(Timer::from_seconds(0.4, TimerMode::Once)));
    commands.spawn((
        AudioPlayer::new(sounds.hit.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

fn burst(click: On<PointerClick>, player: Query<&Transform, With<Player>>, mut commands: Commands) {
    let Ok(transform) = player.get(click.entity) else {
        return;
    };
    // Eight sparks in a ring.
    for i in 0..8 {
        let angle = i as f32 / 8.0 * std::f32::consts::TAU;
        commands.spawn((
            Spark {
                velocity: Vec2::from_angle(angle) * 120.0,
                life: Timer::from_seconds(0.35, TimerMode::Once),
            },
            Sprite::from_color(colour(1), Vec2::splat(5.0)),
            Transform::from_translation(transform.translation.with_z(2.0)),
        ));
    }
}

fn squash(time: Res<Time>, mut query: Query<(&mut Squash, &mut Transform)>) {
    for (mut squash, mut transform) in &mut query {
        squash.0 = (squash.0 - time.delta_secs() * 4.0).max(0.0);
        // Wide and flat on impact, back to square as it fades.
        let s = EaseFunction::QuadraticOut.sample_clamped(squash.0);
        transform.scale = Vec3::new(1.0 + 0.4 * s, 1.0 - 0.35 * s, 1.0);
    }
}

fn blink(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Blink, &mut Visibility)>,
    mut commands: Commands,
) {
    for (entity, mut blink, mut visibility) in &mut query {
        blink.0.tick(time.delta());
        // Off for one twentieth of a second, on for the next.
        let off = blink.0.elapsed_secs() * 20.0 % 2.0 < 1.0;
        *visibility = if off && !blink.0.is_finished() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if blink.0.is_finished() {
            commands.entity(entity).remove::<Blink>();
        }
    }
}

fn sparks(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Spark, &mut Transform)>,
    mut commands: Commands,
) {
    for (entity, mut spark, mut transform) in &mut query {
        spark.life.tick(time.delta());
        transform.translation += (spark.velocity * time.delta_secs()).extend(0.0);
        spark.velocity *= (1.0 - 6.0 * time.delta_secs()).max(0.0); // drag: fast out, then hang
        transform.scale = Vec3::splat(1.0 - spark.life.fraction());
        if spark.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
