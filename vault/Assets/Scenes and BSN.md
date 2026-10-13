---
icon: layers
tags: [assets, scenes, ui, intermediate]
---
# Scenes and BSN

**BSN**[^1] (*Bevy Scene Notation*) describes an entity, its components and its children in one place. It is new in Bevy 0.20 and is how the official examples build 3D scenes and UI. #scenes #ui

> [!NOTE]
> You can always spawn with `commands.spawn((A, B))`. BSN is for when you spawn *trees*, reuse pieces, or want less boilerplate. The two mix freely.

## The `bsn!` macro

Components are separated by whitespace, and children by `--`. Spawn a scene with `commands.spawn_scene(...)`:

```rust preview=menu
use bevy::prelude::*;

fn menu() -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
        Children [
            Text("Play")
            --
            Text("Quit")
        ]
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(menu());
}
```

The preview above is that exact scene, running. Two things to notice:

- `Node { flex_direction: ..., row_gap: ... }` only names two fields. Everything else keeps its default. That is **patching**.
- `Children [ ... ]` spawns each `--`-separated entry as a child.

## Syntax cheat sheet

| Write | Meaning |
|:--|:--|
| `Health(10)` | a component, with a value |
| `Player` | a component, with its `Default` |
| `#Hero` | name the entity (`Name("Hero")`) and make it referable |
| `@menu()` | include another scene (all its parts apply here) |
| `Children [ A -- B ]` | two child entities |
| `Transform::from_xyz(0., 1., 0.)` | any expression that returns a component |
| `{ expression }` | a Rust expression as a value |
| `"player.png"` | for a `Handle` field: load that asset path |
| `on(\|e: On<Ev>\| { ... })` | attach an observer ([[Events and observers]]) |

Which types can appear? Anything with `Default + Clone`. For components that hold handles or entity ids, derive `FromTemplate` instead.

## Patching: compose by layering

```rust preview=wide-menu cont
fn wide_menu() -> impl Scene {
    bsn! {
        @menu()
        Node { width: px(400) }
    }
}
```

`@menu()` brings in the whole menu, then `Node { width: px(400) }` changes only the width. Layers apply in order, so the last one wins on any field it mentions. This is how you make a *variant* of something without copying it: a "danger" button from a button, a "boss" from an enemy.

```rust
use bevy::prelude::*;

#[derive(Component, FromTemplate)]
struct Health { current: u32, max: u32 }

fn enemy() -> impl Scene {
    bsn! { Health { current: 100, max: 100 } }
}

fn boss() -> impl Scene {
    bsn! {
        @enemy()
        Health { max: 500 }       // current stays 100
    }
}
```

## A 3D scene, the 0.20 way

This is Bevy's own `3d_scene` example. `bsn_list!` is a list of root entities, `asset_value` turns a value into an asset handle, and `scene.spawn()` runs it at startup:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, scene.spawn())
        .run();
}

fn scene() -> impl SceneList {
    bsn_list! {
        #CircularBase
        Mesh3d(asset_value(Circle::new(4.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        --
        #Cube
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, 0.0)
        --
        PointLight { shadow_maps_enabled: true }
        Transform::from_xyz(4.0, 8.0, 4.0)
        --
        Camera3d
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
    }
}
```

Compare with [[Meshes and materials]], which builds the same scene with `commands.spawn`.

## Names and references

`#Name` names an entity. Within one `bsn!`, other parts can refer to it, even before it is spawned:

```rust
use bevy::prelude::*;

#[derive(Component, FromTemplate)]
struct Follows(Entity);

fn pair() -> impl SceneList {
    bsn_list! {
        #Leader
        --
        #Follower Follows(#Leader)
    }
}
```

## Spawning

| Call | Does |
|:--|:--|
| `commands.spawn_scene(scene)` | spawn now (errors if an asset it needs is not loaded yet) |
| `commands.queue_spawn_scene(scene)` | wait for assets, then spawn |
| `world.spawn_scene(scene)` | same, with a `World` |
| `system.spawn()` on a fn returning a scene | a ready-made startup system |

## Scenes for UI

Because UI is a tree of entities, BSN fits it best: see [[UI layout]] and the real thing, [[UI kit]], which this very app is built with. Libraries such as `bevy_radioactive` add reactive signals on top of BSN with `view!`.

> [!IMPORTANT]
> The `.bsn` asset *file* format is still in progress; the macro form is the supported way.

## Next

[[Saving and loading]] for game saves, [[Meshes and materials]] or [[UI layout]] to use scenes for real.

[^1]: BSN is pronounced "bison".
