---
icon: square-function
tags: [ecs, beginner]
---
# Systems

A system is a normal Rust function. Its **parameters** say what data it needs; Bevy fetches that data and calls the function. #ecs #beginner

## Your first system

Run it: each system is one step. `spawn_player` runs once, then `regenerate` again and again. Press play or step with the arrows.

```rust example=ecs fps=4
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, Health(50), Transform::default()));
}

fn regenerate(time: Res<Time>, mut query: Query<&mut Health, With<Player>>) {
    for mut health in &mut query {
        health.0 = (health.0 + 1).min(100);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, regenerate)
        .run();
}
```

## Parameters

Common ones:

| Parameter | Gives you | See |
|:--|:--|:--|
| `Query<...>` | entities with certain components | [[Queries]] |
| `Res<T>` / `ResMut<T>` | a global resource | [[Resources]] |
| `Commands` | queue spawns, despawns, inserts | [[Commands]] |
| `MessageReader<M>` / `MessageWriter<M>` | read or send messages | [[Messages]] |
| `Local<T>` | data private to this system | [[System params]] |
| `Single<D>` | exactly one match | [[Advanced queries]] |
| `On<E>` | the event of an observer | [[Events and observers]] |

Order of parameters does not matter. Bevy checks them when the app starts: asking for the same data mutably twice panics with a clear message.

## Parallel by default

Systems that do not touch the same data run at the same time on different threads. `regenerate` and a `mana` system touching `Mana` do not conflict, so Bevy runs them together. You get that for free from the parameter types. Read-only access (`&T`, `Res<T>`) can be shared; mutable access (`&mut T`, `ResMut<T>`) is exclusive.

## Adding systems

```rust
use bevy::prelude::*;

fn a() {}
fn b() {}
fn c() {}

fn main() {
    App::new()
        .add_systems(Startup, a)             // once
        .add_systems(Update, (b, c))         // every frame, any order
        .add_systems(Update, (b, c).chain()) // b, then c
        .run();
}
```

Which schedule you pick decides *when* they run: [[Schedules and sets]]. Skip a system when it should not run: [[Run conditions]].

## Systems that return something

A system can return a [`Result`]; if it returns an error Bevy reports it instead of crashing. See [[Error handling]].

```rust
use bevy::prelude::*;

fn find_player(query: Query<&Transform, With<Camera>>) -> Result {
    let camera = query.single()?;
    info!("camera at {}", camera.translation);
    Ok(())
}
```

## Exclusive systems

A system with `&mut World` can do anything, but nothing else runs beside it. Use it rarely, for things like saving the game.

```rust
use bevy::prelude::*;

fn save_game(world: &mut World) {
    let count = world.entities().len();
    info!("{count} entities");
}
```

## Next

[[Queries]]: the parameter you will use most.
