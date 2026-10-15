---
parent: Hello Bevy
icon: gamepad-2
tags: [start]
crates: rand="0.10"
---
# Make a game

Time to put it all together. **Dodge**: a ship slides along the bottom, rocks fall, you have three lives. Each step adds one idea, and the preview above each block plays the game *so far*. #start

> [!TIP]
> Click in a preview to steer. Out of lives? Click again to restart. The brush, box and speaker buttons swap in your own ship, rocks and hit sound.

## The plan

| Step | Idea | You need |
|:--|:--|:--|
| 1. The ship | spawn entities | [[Entities and components]], [[Commands]] |
| 2. Steering | react to a click, move smoothly | [[Events and observers]], [[Systems]] |
| 3. Rocks | spawn on a timer, move, despawn | [[Time and timers]], [[Queries]] |
| 4. Hits and lives | collisions, a counter | [[Resources]] |
| 5. Game over | stop, show, restart | [[Run conditions]] |

## 1. The ship

The ship is an entity with a `Player` marker, a `TargetX` (where it's heading) and a `Sprite`. The lives are three squares in the corner.

```rust example=dodge
use bevy::prelude::*;

const SIZE: Vec2 = Vec2::new(320.0, 160.0);       // the playfield; the middle is (0, 0)
const FLOOR: f32 = -SIZE.y / 2.0 + 14.0;
const PLAYER: Vec2 = Vec2::new(28.0, 12.0);
const ROCK: f32 = 14.0;

#[derive(Component)]
struct Player;

/// Where the ship is sliding to.
#[derive(Component)]
struct TargetX(f32);

#[derive(Component)]
struct Pip(u32);                                   // life number 0, 1, 2

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        TargetX(0.0),
        Sprite::from_color(Color::srgb(0.35, 0.75, 0.9), PLAYER),
        Transform::from_xyz(0.0, FLOOR, 1.0),
    ));
    for i in 0..3 {
        commands.spawn((
            Pip(i),
            Sprite::from_color(Color::srgb(0.95, 0.45, 0.4), Vec2::splat(8.0)),
            Transform::from_xyz(-SIZE.x / 2.0 + 12.0 + i as f32 * 12.0, SIZE.y / 2.0 - 12.0, 2.0),
        ));
    }
}
```

`Sprite::from_color` is a plain rectangle: all this game needs ([[Sprites]]).

## 2. Steering

A click sets `TargetX`; a system slides the ship toward it. The observer only *records* the wish, the system *acts* on it every frame, at a speed that doesn't depend on the frame rate ([[Time and timers]]).

```rust example=dodge
fn set_target(click: On<PointerClick>, mut player: Single<&mut TargetX, With<Player>>) {
    if let Some(at) = click.hit.position {
        player.0 = at.x;
    }
}

fn slide(time: Res<Time>, mut player: Single<(&mut Transform, &TargetX), With<Player>>) {
    let (transform, target) = &mut *player;
    let step = 140.0 * time.delta_secs();                       // at most this far this frame
    let delta = (target.0 - transform.translation.x).clamp(-step, step);
    transform.translation.x += delta;                            // never overshoots
}

fn add_steering(app: &mut App) {
    app.add_systems(Startup, spawn_player)
        .add_systems(Update, slide)
        .add_observer(set_target);
}
```

`clamp(-step, step)` is the trick: close to the target it moves exactly the rest of the way, so the ship stops *on* it instead of jittering.

## 3. Rocks

A repeating `Timer` drops a rock every 0.6 s at a random x. Another system moves rocks down and despawns the ones that fall off ([[Commands]]).

```rust example=dodge
#[derive(Component)]
struct Rock;

#[derive(Resource)]
struct Spawner(Timer);

#[derive(Resource)]
struct Score(u32);

fn spawn_rocks(time: Res<Time>, mut spawner: ResMut<Spawner>, mut commands: Commands) {
    if spawner.0.tick(time.delta()).just_finished() {
        let x = rand::random_range(-(SIZE.x - ROCK) / 2.0..(SIZE.x - ROCK) / 2.0);
        commands.spawn((
            Rock,
            Sprite::from_color(Color::srgb(0.95, 0.45, 0.4), Vec2::splat(ROCK)),
            Transform::from_xyz(x, SIZE.y / 2.0 + ROCK, 1.0),
        ));
    }
}

fn fall(
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut rocks: Query<(Entity, &mut Transform), With<Rock>>,
    mut commands: Commands,
) {
    for (rock, mut transform) in &mut rocks {
        transform.translation.y -= 80.0 * time.delta_secs();
        if transform.translation.y < -SIZE.y / 2.0 - ROCK {
            commands.entity(rock).despawn();               // left the screen
            score.0 += 1;                                  // dodged one
        }
    }
}

fn add_rocks(app: &mut App) {
    app.insert_resource(Spawner(Timer::from_seconds(0.6, TimerMode::Repeating)))
        .insert_resource(Score(0))
        .add_systems(Startup, spawn_player)
        .add_systems(Update, (slide, spawn_rocks, fall));
}
```

Without the despawn, rocks would fall forever and slowly eat the frame rate. Each rock that falls off is a point.

## 4. Hits and lives

Two boxes overlap when their centres are closer than half their sizes added together, on **both** axes. A hit removes the rock and a life and plays a sound ([[Audio]]).

```rust example=dodge
#[derive(Resource)]
struct Lives(u32);

#[derive(Resource)]
struct Sounds {
    hit: Handle<AudioSource>,
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sounds { hit: asset_server.load("sounds/hit.ogg") });
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
        if apart.x < (PLAYER.x + ROCK) / 2.0 && apart.y < (PLAYER.y + ROCK) / 2.0 {
            commands.entity(rock).despawn();
            lives.0 = lives.0.saturating_sub(1);         // never below zero
            commands.spawn((AudioPlayer::new(sounds.hit.clone()), PlaybackSettings::DESPAWN));
        }
    }
}

fn show_lives(lives: Res<Lives>, mut pips: Query<(&Pip, &mut Sprite)>) {
    for (pip, mut sprite) in &mut pips {
        sprite.color = if pip.0 < lives.0 {
            Color::srgb(0.95, 0.45, 0.4)
        } else {
            Color::srgb(0.25, 0.25, 0.3)               // a lost life goes dark
        };
    }
}

fn add_hits(app: &mut App) {
    app.insert_resource(Lives(3))
        .insert_resource(Spawner(Timer::from_seconds(0.6, TimerMode::Repeating)))
        .insert_resource(Score(0))
        .add_systems(Startup, (spawn_player, load_sounds))
        .add_systems(Update, (slide, spawn_rocks, fall, hit_player, show_lives))
        .add_observer(set_target);
}
```

Let a few rocks hit and watch the lives go dark. The preview has no files to load, so it plays the hit sound you made (speaker button). Nothing stops yet; that's next.

> [!NOTE]
> Hand-written box overlap is perfect for a game this small. For bounce, gravity or odd shapes, see [[Physics]].

## 5. Game over, and again

At zero lives the game should freeze. A **run condition** on the systems that move things does it in one place; a click restarts.

```rust example=dodge
fn alive(lives: Res<Lives>) -> bool {
    lives.0 > 0
}

fn restart(
    _click: On<PointerClick>,
    mut lives: ResMut<Lives>,
    mut score: ResMut<Score>,
    rocks: Query<Entity, With<Rock>>,
    mut commands: Commands,
) {
    if lives.0 == 0 {
        lives.0 = 3;
        score.0 = 0;
        for rock in &rocks {
            commands.entity(rock).despawn();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Lives(3))
        .insert_resource(Score(0))
        .insert_resource(Spawner(Timer::from_seconds(0.6, TimerMode::Repeating)))
        .add_systems(Startup, (|mut commands: Commands| { commands.spawn(Camera2d); }, spawn_player, load_sounds))
        .add_systems(
            Update,
            (slide, spawn_rocks, fall, hit_player).run_if(alive),   // frozen when dead
        )
        .add_systems(Update, show_lives)
        .add_observer(set_target)
        .add_observer(restart)
        .run();
}
```

To add a pause or a title screen later, change that one condition, or move up to [[States]].

## Where to go from here

That's every piece of a real game: input, movement, spawning, timers, collisions, lives, game over. To grow it:

| Add | Learn |
|:--|:--|
| a picture instead of a rectangle | [[Sprites]], [[Sprite animation]] |
| sound on hit and dodge | [[Audio]] |
| the score on screen | [[Text 2D]] or [[UI]] |
| harder over time | make the timer shorter as `Score` grows |
| a title screen and pause | [[States]] |
| screen shake when hit | [[Game feel]] |
| keyboard or gamepad | [[Input#Keyboard and mouse\|Keyboard and mouse]], [[Input mapping]] |
| ship it | [[Web build]], [[Release builds]] |

> [!NOTE]
> `Score` already counts; it just isn't on screen yet. [[Text 2D]] shows it in five lines.
