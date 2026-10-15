---
icon: bug
tags: [tooling]
---
# Debugging

A short field guide: what to try when the game does the wrong thing. #tooling

## "My screen is black / nothing shows"

Walk the list:

- [ ] **A camera?** `Camera2d` or `Camera3d` must exist, one active per target ([[Cameras]]).
- [ ] **3D without a light?** It's black ([[Lighting]]).
- [ ] **Out of view?** Behind the camera, too far, or the wrong `z` in 2D. Print the `Transform` ([[Transforms]]).
- [ ] **Not loaded?** Check the path and the log for "file not found" ([[Assets]]).
- [ ] **Hidden?** `Visibility::Hidden` on it or an ancestor, or `Display::None` in UI.
- [ ] **Zero-sized?** Scale 0, alpha 0, or `custom_size: Some(Vec2::ZERO)`.
- [ ] **Feature off?** No `DefaultPlugins`, or a Cargo feature missing ([[Project setup]]).

## "My system never runs"

- Is it **added** (`.add_systems`)? In the right **schedule** ([[Schedules and sets]])?
- A [[Run conditions|run condition]] false? A state not entered ([[States]])?
- A `Single<...>` or `Populated<...>` parameter with no match **silently skips** the system ([[Error handling]]). Try `Query` and `.single()` while debugging to see an error.
- `Query` matches nothing because a required component is missing on the entity: print the entity's components.

## Look at the world

Print what an entity has:

```rust
use bevy::prelude::*;

fn what_is_it(world: &World, entity: Entity) {
    let components: Vec<_> = world
        .inspect_entity(entity)
        .unwrap()
        .map(|(_, info)| info.name().to_string())
        .collect();
    info!("{entity} has {components:?}");
}
```

Or use an inspector that shows the whole world and lets you edit it live: [[bevy-inspector-egui]]. For external tools use the remote protocol ([[Diagnostics and logging#Dev tools|Dev tools]]).

## See what the schedule does

Draw the system graph with [[bevy_mod_debugdump]], or step one system at a time with Bevy's `Stepping`: the same feature the previews on this site use.

## Bugs with time and order

- Behaves differently at different frame rates → you forgot `delta_secs()` ([[Time and timers]]).
- Something is one frame late → system order. Add `.chain()` or `.after(...)`.
- Commands applied "too late" → they run when the system ends, not at the call ([[Commands]]).

## Panics

Read the **first** lines of the message. Typical Bevy ones:

| Message | Cause |
|:--|:--|
| `Resource requested by ... does not exist` | forgot `insert_resource` / `init_resource` |
| `error[B0001]: Query ... conflicts` | two queries borrow the same component mutably; add `Without<>` or use `ParamSet` ([[Advanced queries]]) |
| `Entity ... does not exist` | the entity was despawned; use `if let Ok(...) = query.get(...)` |
| `Error parsing...` of assets | wrong asset format or missing Cargo feature |

Set `RUST_BACKTRACE=1` to see where. [[Diagnostics and logging]] has `RUST_LOG`.

## Make bugs visible

Draw them with [[Gizmos]]; show numbers on screen with [[Diagnostics and logging#Dev tools|Dev tools]]; write a [[Testing|test]] that reproduces it.

## Next

[[Hot reloading]] to iterate faster.
