---
icon: database
tags: [ecs]
---
# Resources

A **resource** is data there is only one of: the score, the settings, the clock. Entities have many components; the world has one `Score`. #ecs

## Define and insert

```rust example=resources fps=3
use bevy::prelude::*;

#[derive(Resource)]
struct Score {
    points: u32,
}

#[derive(Component)]
struct Coin;

fn count_coins(mut score: ResMut<Score>, coins: Query<&Coin>) {
    score.points = coins.iter().count() as u32 * 10;
}

fn main() {
    App::new()
        .insert_resource(Score { points: 0 })   // with a value
        .add_systems(Update, count_coins)
        .run();
}
```

The card with `Score` is the resource itself: since Bevy 0.20 a resource is a component on a hidden singleton entity. Press **Collect a coin** and step to see the score follow.

## Reading and writing

| Parameter | Access |
|:--|:--|
| `Res<Score>` | read |
| `ResMut<Score>` | read and write |
| `Option<Res<Score>>` | read, if it exists |

A system asking for `Res<Score>` when none was inserted panics, so insert your resources before running. Use `Option<Res<T>>` for things that may be missing.

## Default values

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Settings {
    volume: f32,
    fullscreen: bool,
}

#[derive(Resource)]
struct Gravity(f32);

impl Default for Gravity {
    fn default() -> Self {
        Gravity(9.8)
    }
}

fn main() {
    App::new()
        .init_resource::<Settings>()   // uses Default
        .init_resource::<Gravity>()
        .run();
}
```

`init_resource` does nothing if the resource already exists, so plugins can call it safely.

## Resources you already use

[`Time`], [`AssetServer`], `Assets<T>`, `ButtonInput<KeyCode>` (the keyboard), [`ClearColor`], `Window` data... most of Bevy's own state is a resource.

## Resource or component?

- One of it, global → resource (`Score`, `Settings`, `Gravity`).
- Belongs to a thing → component (`Health`, `Team`).
- "The player's score" while there is one player → either. If you might add a second player later, use a component.

## Change detection

`ResMut<T>` knows whether it was written: check `score.is_changed()` to react once per change.

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct Score(u32);

fn show_score(score: Res<Score>) {
    if score.is_changed() {
        info!("score is now {}", score.0);
    }
}
```

More in [[Change detection]].

## Next

[[Events and observers]]: telling the rest of the game that something happened.
