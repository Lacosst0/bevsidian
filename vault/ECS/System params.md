---
icon: sliders-horizontal
tags: [ecs]
---
# System params

Beyond `Query`, `Res` and `Commands`: the other things a system can ask for. #ecs

## Local

Data that belongs to one system and lives as long as the app. No resource needed:

```rust
use bevy::prelude::*;

fn count_frames(mut frames: Local<u32>) {
    *frames += 1;
    if *frames % 60 == 0 {
        info!("{} frames", *frames);
    }
}
```

`Local<T>` starts at `T::default()`. Two copies of the same system each get their own.

## ParamSet

Two queries that might touch the same entity mutably cannot be in the same system. A `ParamSet` lets you use them one at a time:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy;

fn swap_positions(mut set: ParamSet<(
    Query<&mut Transform, With<Player>>,
    Query<&mut Transform, With<Enemy>>,
)>) {
    let player = set.p0().single().map(|t| t.translation).ok();
    if let (Some(player), Ok(mut enemy)) = (player, set.p1().single_mut()) {
        enemy.translation = player;
    }
}
```

## Your own SystemParam

Wrap a group of parameters you always use together:

```rust
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;

#[derive(Component)] struct Health(u32);
#[derive(Resource)] struct Difficulty(f32);

#[derive(SystemParam)]
struct Combat<'w, 's> {
    healths: Query<'w, 's, &'static mut Health>,
    difficulty: Res<'w, Difficulty>,
}

impl Combat<'_, '_> {
    fn hurt(&mut self, entity: Entity, amount: u32) {
        let amount = (amount as f32 * self.difficulty.0) as u32;
        if let Ok(mut health) = self.healths.get_mut(entity) {
            health.0 = health.0.saturating_sub(amount);
        }
    }
}

fn attack(mut combat: Combat) {
    // combat.hurt(some_entity, 10);
}
```

## Closures as systems

Capture local values; handy in plugins that take settings:

```rust
use bevy::prelude::*;

fn main() {
    let message = String::from("hello");
    App::new()
        .add_systems(Startup, move || info!("{message}"))
        .run();
}
```

## Passing data between systems: pipe

`a.pipe(b)` feeds the output of `a` into `b`. Typical use: turn a `Result` into a log line.

```rust
use bevy::prelude::*;

fn load() -> Result<u32> {
    Ok(42)
}

fn report(In(result): In<Result<u32>>) {
    match result {
        Ok(n) => info!("loaded {n}"),
        Err(e) => error!("{e}"),
    }
}

fn main() {
    App::new().add_systems(Startup, load.pipe(report)).run();
}
```

## One-shot systems

A system you run **on demand**, not every frame: register it, then call it by id.

```rust
use bevy::prelude::*;
use bevy::ecs::system::SystemId;

#[derive(Resource)]
struct Actions { save: SystemId }

fn save_game() { info!("saved"); }

fn setup(world: &mut World) {
    let save = world.register_system(save_game);
    world.insert_resource(Actions { save });
}

fn on_key(keys: Res<ButtonInput<KeyCode>>, actions: Res<Actions>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::KeyS) {
        commands.run_system(actions.save);
    }
}
```

Use them for "menu button runs this" or "console command". Observers and [[Events and observers|events]] are usually a better fit, but one-shot systems are great when the action has no natural event.

## More built-ins

| Parameter | For |
|:--|:--|
| `Commands` | [[Commands]] |
| `Query` / `Single` | [[Queries]] |
| `Res` / `ResMut` | [[Resources]] |
| `MessageReader` / `MessageWriter` | [[Messages]] |
| `Option<P>` | `P`, if it can be had (e.g. a resource that may not exist) |
| `Populated<D, F>` | a query that **skips the system** while it is empty |
| `NonSend<T>` | main-thread-only data (rare) |
| `&mut World` | everything (exclusive) |
| `Time`, `Assets<T>`, `AssetServer` | ordinary resources |

`Populated` is a nice shortcut:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Enemy;

fn chase(enemies: Populated<&mut Transform, With<Enemy>>) {
    // only runs when there is at least one Enemy
}
```

## Next

[[Time and timers]].
