---
icon: rocket
tags: [bsn, scenes, intermediate]
---
# Spawning scenes

You have a scene. Here is every way to put it in the world, and which to pick. #bsn #scenes

## Now or when ready

| Call | Does |
|:--|:--|
| `commands.spawn_scene(scene)` | resolves and spawns **now**; with `Commands`, errors are logged |
| `world.spawn_scene(scene)` | same on a `World`; returns a `Result` |
| `commands.queue_spawn_scene(scene)` | **waits** for the assets the scene needs, then spawns |
| `world.queue_spawn_scene(scene)` | same on a `World` |

The difference matters when the scene loads files. `spawn_scene` fails with an error if a dependency is not loaded yet; `queue_spawn_scene` registers what it needs, waits, and spawns during the `SpawnScene` schedule (between `Update` and `PostUpdate`) on the frame everything is ready.

```rust
use bevy::prelude::*;

fn player() -> impl Scene {
    bsn! { Sprite { image: "player.png" } }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.queue_spawn_scene(player());         // waits for player.png
}
```

Both return the root entity's `EntityCommands`, so you can keep going:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Hero;

fn setup(mut commands: Commands) {
    commands
        .spawn_scene(bsn! { Name("Hero") Transform::from_xyz(0.0, 0.0, 0.0) })
        .insert(Hero);
}
```

## Many roots: scene lists

`bsn_list!` produces a `SceneList`: one entity per `--`.

| Call | Does |
|:--|:--|
| `commands.spawn_scene_list(list)` | spawn every root now |
| `commands.queue_spawn_scene_list(list)` | wait for assets, then spawn every root |
| `world.spawn_scene_list(list)` | same on a `World`; returns the entities |

## A startup system for free

A function that returns a scene or a scene list becomes a system with `.spawn()`:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, world_scene.spawn())    // a SceneList
        .add_systems(Startup, hud.spawn())            // a Scene
        .run();
}

fn world_scene() -> impl SceneList {
    bsn_list! { Camera2d -- Sprite { image: "player.png" } }
}

fn hud() -> impl Scene {
    bsn! { Text("Score: 0") }
}
```

It is meant for schedules that run once, such as `Startup`. Put it in `Update` and it spawns a new copy every frame.

## Into an existing entity

| Call | Does |
|:--|:--|
| `entity.apply_scene(scene)` | writes the scene **on top of** an entity that already exists |
| `entity.queue_apply_scene(scene)` | same, after assets load |
| `entity.queue_spawn_related_scenes::<Children>(list)` | spawn the list as that entity's children (or any relationship) |

```rust
use bevy::prelude::*;

fn rebuild(mut commands: Commands, targets: Query<Entity, With<Name>>) {
    for entity in &targets {
        commands.entity(entity).apply_scene(bsn! { Text("rebuilt") });
    }
}

fn add_items(mut commands: Commands, bag: Single<Entity, With<Name>>) {
    commands.entity(*bag).queue_spawn_related_scenes::<Children>(bsn_list! {
        Name("apple") -- Name("pear")
    });
}
```

> [!WARNING]
> Applying a scene **replaces and orphans** the entity's children. That is how `bevy_radioactive`'s hooks re-render: despawn the children, apply the new scene.

## Scenes in Bevy's other systems

- A plain `commands.spawn(...)` and a scene mix freely; BSN is not a separate world.
- BSN is for **writing** scenes in code. To **save** a world to a file, use `DynamicWorld` ([[Saving and loading]]); to load a model, use glTF ([[Assets]]).
- The `.bsn` file format will let scenes live on disk and hot-reload. It is not in 0.20, but the loader architecture exists, so community crates can provide it.

## Common mistakes

| Symptom | Cause |
|:--|:--|
| error "asset not loaded" at startup | use `queue_spawn_scene` instead of `spawn_scene` |
| the scene appears every frame | `.spawn()` or `spawn_scene` in `Update` |
| `trait bound ... FromTemplate` | a field holds a `Handle` or `Entity`; derive `FromTemplate` ([[Templates and assets]]) |
| scene component logs an error | spawned with `commands.spawn`; use `@Type` ([[Scene components]]) |
| enum field "missing fields" | enums are written in full ([[BSN syntax]]) |

## Next

Use it: [[UI layout]] builds interfaces from scenes, and [[bevy_radioactive]] makes them react.
