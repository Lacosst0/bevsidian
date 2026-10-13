---
icon: flask-conical
tags: [ecs, intermediate]
---
# Testing

Because systems are plain functions and the world is plain data, tests are easy: build a world, run a system, check the result. #ecs #intermediate

## Test a system

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);

fn heal(mut query: Query<&mut Health>) {
    for mut health in &mut query {
        health.0 += 1;
    }
}

#[test]
fn heal_adds_one() {
    let mut app = App::new();
    app.add_systems(Update, heal);

    let hero = app.world_mut().spawn(Health(10)).id();
    app.update();                        // run one frame

    assert_eq!(app.world().get::<Health>(hero).unwrap().0, 11);
}
```

`App::new()` with no plugins is fast and has no window, no GPU. Add only what the code needs (`app.init_resource::<...>()`, `app.add_plugins(MinimalPlugins)` for time, `StatesPlugin` for states).

## Test a world directly

No app at all, run one system once:

```rust
use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;

#[derive(Component)] struct Coin;

fn spawn_coins(mut commands: Commands) {
    for _ in 0..3 { commands.spawn(Coin); }
}

#[test]
fn spawns_three_coins() {
    let mut world = World::new();
    world.run_system_once(spawn_coins).unwrap();
    assert_eq!(world.query::<&Coin>().iter(&world).count(), 3);
}
```

## Test with time

```rust
use bevy::prelude::*;
use std::time::Duration;

#[test]
fn time_advances() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.world_mut().resource_mut::<Time<Virtual>>().advance_by(Duration::from_secs(1));
    app.update();
}
```

## Tips

- Keep logic in small systems and plugins, so a test can add just that plugin ([[App and plugins]]).
- Test observers by `world.trigger(Event)` then read the world.
- Visual features (rendering, UI layout) are better checked by eye, or with screenshot tests via the `bevy_ci_testing` features.
- `cargo test` compiles Bevy in debug; the dev profile in [[Project setup]] keeps it fast.

## Next

You have the whole ECS. Put it to work in [[A 2D game]], or pick an area from [[Bevy]].
