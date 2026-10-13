---
icon: triangle-alert
tags: [ecs, intermediate]
---
# Error handling

Systems can fail: the entity you wanted is gone, a file is missing. Instead of `unwrap()` crashing the game, return a `Result`. #ecs #intermediate

## Fallible systems

A system may return `Result` (Bevy's alias for `Result<(), BevyError>`). Use `?` freely:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player;
#[derive(Component)] struct Health(u32);

fn heal(mut query: Query<&mut Health, With<Player>>) -> Result {
    let mut health = query.single_mut()?;   // error if there is not exactly one player
    health.0 += 1;
    Ok(())
}
```

If it returns `Err`, Bevy's **error handler** runs. By default that **panics** with a clear message, so you notice during development.

## Choose what happens

```rust
use bevy::prelude::*;
use bevy::ecs::error::warn;

fn main() {
    App::new()
        // Log a warning and carry on, instead of panicking.
        .set_error_handler(warn)
        .run();
}
```

Handlers available: `panic` (default), `warn`, `error`, `info`, `debug`, `trace`, `ignore`; or write your own function taking a `BevyError` and its `ErrorContext` (which system failed, and where). A good release setting is `warn` or `error`; keep `panic` while developing.

## Fallible parameters

Some parameters may not be available. By default a system with an unavailable parameter **panics**, but you can choose:

| Type | If missing |
|:--|:--|
| `Single<D>` | skips the system quietly |
| `Option<Single<D>>` | gives you `None` |
| `Res<T>` | panics (`Option<Res<T>>` for `None`) |
| `If<P>` | skips the system if `P` is unavailable |

```rust
use bevy::prelude::*;

#[derive(Component)] struct Boss;

fn show_boss_bar(boss: Option<Single<&Transform, With<Boss>>>) {
    match boss {
        Some(boss) => info!("boss at {}", boss.translation),
        None => info!("no boss yet"),
    }
}
```

## Your own errors

```rust
use bevy::prelude::*;

fn load_level() -> Result {
    let text = std::fs::read_to_string("assets/level.txt")?;   // io error → BevyError
    let width: u32 = text.lines().next().ok_or("empty level")?.parse()?;
    info!("level is {width} wide");
    Ok(())
}
```

Anything implementing `std::error::Error` converts with `?`; a `&str` works too.

## Observers can fail as well

```rust
use bevy::prelude::*;

#[derive(EntityEvent)] struct Pick { entity: Entity }
#[derive(Component)] struct Item;

fn on_pick(pick: On<Pick>, items: Query<&Item>) -> Result {
    items.get(pick.entity)?;
    Ok(())
}
```

## Commands that can fail

`commands.entity(e).despawn()` panics in old Bevy if `e` is gone; today it logs through the error handler instead. For a quiet try use `try_despawn`/`try_insert`.

## Tips

- During development `panic` is your friend: it shows exactly what you got wrong.
- Use `Single` for "there should be one" and let the system skip while it does not exist yet.
- Do not use `unwrap()` on query results; `?` is shorter and reports better.

## Next

[[Component lifecycle]].
