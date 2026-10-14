---
parent: Systems
icon: calendar
tags: [ecs]
---
# Schedules and sets

A **schedule** is a list of systems that run together. Bevy runs a fixed set of them every frame, in a fixed order. You pick *when* a system runs by picking its schedule. #ecs

## One frame

```text
Startup (once) → [ every frame ]  First → PreUpdate → StateTransition → RunFixedMainLoop → Update → PostUpdate → Last
```

| Schedule | Runs | Use it for |
|:--|:--|:--|
| `PreStartup`, `Startup`, `PostStartup` | once, at the start | spawning the first things |
| `First`, `PreUpdate` | every frame, early | library work before your code |
| **`Update`** | every frame | **your game logic** |
| `PostUpdate`, `Last` | every frame, late | work that must see your changes (transforms, rendering prep) |
| `FixedUpdate` | at a fixed rate (64 Hz) | physics, AI, game rules ([[Time and timers]]) |
| `OnEnter(State)`, `OnExit(State)` | on a state change | [[States]] |

When in doubt: `Startup` for setup, `Update` for everything else.

## Order inside a schedule

Systems in one schedule run **in parallel, in no fixed order**. If order matters, say so:

```rust
use bevy::prelude::*;

fn read_input() {}
fn move_player() {}
fn animate() {}
fn camera_follow() {}

fn main() {
    App::new()
        .add_systems(
            Update,
            (
                (read_input, move_player, animate).chain(),   // one after another
                camera_follow.after(move_player),             // after one system
            ),
        )
        .run();
}
```

- `.chain()` runs a tuple in the order written.
- `.before(x)` / `.after(x)` order against one system or set.
- Only order what must be ordered: unordered systems run in parallel and the game is faster.

> [!TIP]
> Bevy applies the queued [[Commands]] of a system before the next ordered system starts. So `(spawn, use_spawned).chain()` works.

## System sets

A **set** is a name for a group of systems. Order against the group, not each member.

```rust
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum Phase {
    Input,
    Movement,
    Effects,
}

fn read_keys() {}
fn walk() {}
fn dust() {}

fn main() {
    App::new()
        .configure_sets(Update, (Phase::Input, Phase::Movement, Phase::Effects).chain())
        .add_systems(Update, read_keys.in_set(Phase::Input))
        .add_systems(Update, walk.in_set(Phase::Movement))
        .add_systems(Update, dust.in_set(Phase::Effects))
        .run();
}
```

Now any plugin can put its systems in `Phase::Movement` without knowing about the others. Sets can also share a [[Run conditions|run condition]]: `.configure_sets(Update, Phase::Movement.run_if(in_state(...)))`.

## Your own schedules

Define a label and run it by hand:

```rust
use bevy::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct TurnEnd;

fn end_turn() {}

fn run_turn_end(world: &mut World) {
    world.run_schedule(TurnEnd);
}

fn main() {
    App::new()
        .add_systems(TurnEnd, end_turn)
        .add_systems(Update, run_turn_end)
        .run();
}
```

Useful for turn-based games and simulations: a whole group of systems you trigger exactly when you want.

## Seeing the order

Bevy can draw the schedule for you: see `bevy_mod_debugdump` in [[Debugging]]. The `schedule_data` dev tool (feature `bevy_dev_tools`) lists systems at runtime.

## Next

[[Run conditions]]: skip systems that do not apply. [[States]]: switch whole groups on and off.
