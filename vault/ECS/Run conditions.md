---
icon: list-checks
tags: [ecs, intermediate]
---
# Run conditions

A **run condition** is a function that returns `bool`. Attach it to a system and the system only runs when it returns `true`. #ecs #intermediate

## Attach a condition

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct Paused(bool);

fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

fn move_enemies() {}

fn main() {
    App::new()
        .insert_resource(Paused(false))
        .add_systems(Update, move_enemies.run_if(not_paused))
        .run();
}
```

A condition is a system too: it can take any read-only parameter (`Res`, `Query`, ...).

## Built in

```rust
use bevy::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::time::common_conditions::on_timer;

#[derive(Resource)] struct Score(u32);
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum Mode { #[default] Menu, Playing }
#[derive(Component)] struct Enemy;
#[derive(Message)] struct Hit;

fn a() {} fn b() {} fn c() {} fn d() {} fn e() {} fn f() {} fn g() {}

fn main() {
    App::new()
        .add_systems(Update, (
            a.run_if(in_state(Mode::Playing)),            // see States
            b.run_if(input_just_pressed(KeyCode::Space)), // key pressed this frame
            c.run_if(resource_exists::<Score>),
            d.run_if(resource_changed::<Score>),
            e.run_if(on_message::<Hit>),                  // a message arrived
            f.run_if(any_with_component::<Enemy>),        // at least one Enemy exists
            g.run_if(on_timer(std::time::Duration::from_secs(1))), // once a second
        ))
        .run();
}
```

## Combine them

```rust
use bevy::prelude::*;
use bevy::input::common_conditions::input_just_pressed;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum Mode { #[default] Menu, Playing }

fn shoot() {}

fn main() {
    App::new()
        .add_systems(Update, shoot.run_if(
            in_state(Mode::Playing).and_then(input_just_pressed(KeyCode::Space)),
        ))
        .run();
}
```

| Combinator | True when | Evaluates the second? |
|:--|:--|:--|
| `a.and_then(b)` | both | only if `a` is true |
| `a.and_eager(b)` | both | always |
| `a.or_else(b)` | either | only if `a` is false |
| `a.or_eager(b)` | either | always |
| `a.xor(b)` | exactly one | always |
| `not(a)` | `a` is false | (`bevy::ecs::schedule::common_conditions::not`) |

Prefer `and_then`/`or_else`: they skip work, and a condition that is never evaluated cannot panic on a missing resource. Use the `_eager` forms when both conditions track change detection or messages and must see every frame. Several `.run_if` calls on one system also mean *all must be true*.

## Conditions on many systems

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum Mode { #[default] Menu, Playing }

fn move_player() {}
fn move_enemies() {}

fn main() {
    App::new()
        .add_systems(
            Update,
            (move_player, move_enemies).run_if(in_state(Mode::Playing)),
        )
        .run();
}
```

## Careful with messages

A system that does not run still lets [[Messages]] expire after two frames. If a conditioned system reads messages, it may miss some: use `on_message::<M>` as its condition, or read them in an unconditional system.

## Run conditions on observers

Observers are systems too: `.add_observer((|ev: On<Boom>| { ... }).run_if(...))`.

## Next

[[States]]: the most common use of conditions.
