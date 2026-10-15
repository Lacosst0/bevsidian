//! `Make a game.md`: dodge the falling rocks. Click to slide the ship; three lives.
//!
//! The note uses `rand::random_range`; the preview has a tiny generator of its own, so it
//! needs no crates and plays the same every time.

use bevy::{audio::AudioSource, prelude::*};

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
        startup!(spawn_player),
        observer!(set_target),
        update!(slide),
        update!(spawn_rocks),
        update!(fall),
        update!(hit_player),
        update!(show_lives),
        observer!(restart),
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
        world.insert_resource(Lives(3));
        world.insert_resource(Score(0));
        world.insert_resource(Spawner(Timer::from_seconds(0.6, TimerMode::Repeating)));
        world.insert_resource(Seed(0x2545_F491_4F6C_DD1D));
    })
}

const FLOOR: f32 = -SIZE.y / 2.0 + 14.0;
const PLAYER: Vec2 = Vec2::new(28.0, 12.0);
const ROCK: f32 = 14.0;

#[derive(Resource)]
struct Lives(u32);

#[derive(Resource)]
struct Sounds {
    hit: Handle<AudioSource>,
}

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct Spawner(Timer);

#[derive(Resource)]
struct Seed(u64);

impl Seed {
    /// A number in 0..1.
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 10_000) as f32 / 10_000.0
    }
}

#[derive(Component)]
struct Player;

/// Where the ship is sliding to.
#[derive(Component)]
struct TargetX(f32);

#[derive(Component)]
struct Rock;

#[derive(Component)]
struct Pip(u32);

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        TargetX(0.0),
        Sprite::from_color(colour(3), PLAYER),
        Transform::from_xyz(0.0, FLOOR, 1.0),
    ));
    for i in 0..3 {
        commands.spawn((
            Pip(i),
            Sprite::from_color(colour(0), Vec2::splat(8.0)),
            Transform::from_xyz(
                -SIZE.x / 2.0 + 12.0 + i as f32 * 12.0,
                SIZE.y / 2.0 - 12.0,
                2.0,
            ),
        ));
    }
}

fn set_target(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    lives: Res<Lives>,
    mut player: Single<&mut TargetX, With<Player>>,
) {
    if let (true, Some(at), true) = (sky.contains(click.entity), click.hit.position, lives.0 > 0) {
        player.0 = at.x;
    }
}

fn slide(
    time: Res<Time>,
    lives: Res<Lives>,
    mut player: Single<(&mut Transform, &TargetX), With<Player>>,
) {
    if lives.0 == 0 {
        return;
    }
    let (transform, target) = &mut *player;
    let step = 140.0 * time.delta_secs();
    let delta = (target.0 - transform.translation.x).clamp(-step, step);
    transform.translation.x += delta;
}

fn spawn_rocks(
    time: Res<Time>,
    lives: Res<Lives>,
    mut spawner: ResMut<Spawner>,
    mut seed: ResMut<Seed>,
    mut commands: Commands,
) {
    if lives.0 > 0 && spawner.0.tick(time.delta()).just_finished() {
        let x = (seed.next() - 0.5) * (SIZE.x - ROCK);
        commands.spawn((
            Rock,
            Sprite::from_color(colour(0), Vec2::splat(ROCK)),
            Transform::from_xyz(x, SIZE.y / 2.0 + ROCK, 1.0),
        ));
    }
}

fn fall(
    time: Res<Time>,
    lives: Res<Lives>,
    mut score: ResMut<Score>,
    mut rocks: Query<(Entity, &mut Transform), With<Rock>>,
    mut commands: Commands,
) {
    if lives.0 == 0 {
        return;
    }
    for (rock, mut transform) in &mut rocks {
        transform.translation.y -= 80.0 * time.delta_secs();
        if transform.translation.y < -SIZE.y / 2.0 - ROCK {
            commands.entity(rock).despawn();
            score.0 += 1;
        }
    }
}

fn hit_player(
    player: Single<&Transform, With<Player>>,
    rocks: Query<(Entity, &Transform), With<Rock>>,
    mut lives: ResMut<Lives>,
    sounds: Res<Sounds>,
    mut commands: Commands,
) {
    for (rock, transform) in &rocks {
        let apart = (transform.translation - player.translation).abs();
        if apart.x < PLAYER.x.midpoint(ROCK) && apart.y < PLAYER.y.midpoint(ROCK) {
            commands.entity(rock).despawn();
            lives.0 = lives.0.saturating_sub(1);
            commands.spawn((
                AudioPlayer::new(sounds.hit.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn show_lives(lives: Res<Lives>, mut pips: Query<(&Pip, &mut Sprite)>) {
    for (pip, mut sprite) in &mut pips {
        sprite.color = if pip.0 < lives.0 {
            colour(0)
        } else {
            Color::srgb(0.25, 0.25, 0.3)
        };
    }
}

fn restart(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    mut lives: ResMut<Lives>,
    mut score: ResMut<Score>,
    rocks: Query<Entity, With<Rock>>,
    mut commands: Commands,
) {
    if sky.contains(click.entity) && lives.0 == 0 {
        lives.0 = 3;
        score.0 = 0;
        for rock in &rocks {
            commands.entity(rock).despawn();
        }
    }
}
