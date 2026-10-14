---
parent: Hello Bevy
icon: gamepad-2
tags: [start]
crates: rand="0.10"
---
# Make a game

Time to put it together. We will build **Dodge**: a ship slides left and right, rocks fall, you have three lives. Each step adds one idea from earlier notes, and the preview above each block plays the game *so far*. #start

> [!TIP]
> Click inside a preview to move the ship. When the last life goes, click again to restart.

## The plan

| Step | Idea | You need |
|:--|:--|:--|
| 1. The ship | spawn entities | [[Entities and components]], [[Commands]] |
| 2. Steering | react to a click, move smoothly | [[Events and observers]], [[Systems]] |
| 3. Rocks | spawn on a timer, move, despawn | [[Time and timers]], [[Queries]] |
| 4. Hits and lives | collisions, a counter | [[Resources]] |
| 5. Game over | stop, show, restart | [[Run conditions]] |

## 1. The ship

Everything is an entity. The ship has a `Player` marker, a `TargetX` (where it is sliding to) and a `Sprite`. The lives are three little squares in the corner.

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

A plain `Sprite::from_color` is a coloured rectangle, which is all this game needs ([[Sprites]]).

## 2. Steering

A click should set the ship's `TargetX`; a system slides the ship towards it. Two small functions, one job each. The observer only *records* the wish; the system *acts* on it every frame, at a speed that does not depend on the frame rate (`time.delta_secs()`, see [[Time and timers]]).

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

`clamp(-step, step)` is the trick: it moves the ship the whole way when it is close, so it stops *on* the target instead of jittering around it.

## 3. Rocks

A `Timer` that repeats every 0.6 seconds spawns a rock at a random x at the top. Another system moves every rock down, and removes the ones that left the bottom ([[Commands]]).

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

Without the despawn the rocks would fall forever and the game would slow down. Every rock that leaves the screen is a point.

## 4. Hits and lives

Two boxes overlap when the distance between their centres is less than half of their sizes added together, on **both** axes. A hit removes the rock and a life; the pips on screen show the lives left.

```rust example=dodge
#[derive(Resource)]
struct Lives(u32);

fn hit_player(
    player: Single<&Transform, With<Player>>,
    rocks: Query<(Entity, &Transform), With<Rock>>,
    mut lives: ResMut<Lives>,
    mut commands: Commands,
) {
    for (rock, transform) in &rocks {
        let apart = (transform.translation - player.translation).abs();
        if apart.x < (PLAYER.x + ROCK) / 2.0 && apart.y < (PLAYER.y + ROCK) / 2.0 {
            commands.entity(rock).despawn();
            lives.0 = lives.0.saturating_sub(1);         // never below zero
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
        .add_systems(Startup, spawn_player)
        .add_systems(Update, (slide, spawn_rocks, fall, hit_player, show_lives))
        .add_observer(set_target);
}
```

Let a few rocks hit the ship in the preview and watch the lives go dark. Nothing stops yet: the next step does that.

> [!NOTE]
> This is hand-written box overlap, perfect for a game this small. For anything with more shapes, bounce or gravity use a physics engine: [[Physics]].

## 5. Game over, and again

When `Lives` hits zero, the game should stop: no new rocks, no movement. The cleanest way is a **run condition** on the systems that make the game move. Then a click restarts it.

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
        .add_systems(Startup, (|mut commands: Commands| { commands.spawn(Camera2d); }, spawn_player))
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

`run_if(alive)` puts one rule in one place. Add a pause screen, a menu, or a win condition later by changing that condition ([[States]] are the next step up).

## Where to go from here

You now have every piece of a real game: input, movement, spawning, a timer, collisions, a score and lives, game over and restart. To grow it:

| Add | Learn |
|:--|:--|
| a picture instead of a rectangle | [[Sprites]], [[Sprite animation]] |
| sound on hit and dodge | [[Audio]] |
| the score on screen | [[Text 2D]] or [[UI]] |
| harder over time | make the timer shorter as `Score` grows |
| a title screen and pause | [[States]] |
| screen shake when hit | [[Game feel]] |
| keyboard or gamepad | [[Keyboard and mouse]], [[Input mapping]] |
| ship it | [[Web build]], [[Release builds]] |

> [!IMPORTANT]
> The score exists but is not shown, because text deserves its own note. When you add it, `Score` will already be updating correctly.
