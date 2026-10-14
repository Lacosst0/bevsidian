---
icon: layers
tags: [bsn, ui]
---
# Scenes and BSN

Your first scene. By the end you will have spawned a menu, made a wider copy of it without repeating anything, and seen the same scene style build a 3D world. #bsn

> [!NOTE]
> You can always spawn with `commands.spawn((A, B))`. BSN is for when you spawn *trees*, reuse pieces, or want less boilerplate. The two mix freely. The map of all BSN notes is [[BSN]].

## A scene is a function

A **scene** describes one entity: its components, and the entities under it. First, a small menu in plain Bevy:

```rust
use bevy::prelude::*;

fn menu_plain(mut commands: Commands) {
    commands.spawn((
        Node { flex_direction: FlexDirection::Column, row_gap: px(8), ..default() },
        children![Text::new("Play"), Text::new("Quit")],
    ));
}
```

The plain version is fine for this. BSN pays off when the tree grows, or when you want to reuse it.

Now the same as a scene. You write one with `bsn!` and return it as `impl Scene`. Components are separated by whitespace, children by `--`:

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

- `Node { flex_direction: ..., row_gap: ... }` names two fields. Everything else keeps its default. That is **patching**, and it is the idea the rest of BSN stands on.
- `Children [ ... ]` spawns each `--`-separated entry as a child entity. `Text("Play")` is one entity with a `Text`.


## Reuse by patching

A scene can include another with `@`. The included scene's parts apply first, then yours, so **the last one wins on any field it mentions**:

```rust preview=wide-menu cont
fn wide_menu() -> impl Scene {
    bsn! {
        @menu()
        Node { width: px(400) }
    }
}
```

`@menu()` brings in the whole menu, then `Node { width: px(400) }` changes only the width. The rows, the gap and the direction stay. This is how you make a *variant* without copying: a "danger" button from a button, a "boss" from an enemy.

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

More on this, with parameters and lists, in [[Composing scenes]].

## One entity or many

Components in a row belong to **one** entity. A new `--` starts the next one:

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)] struct A;
#[derive(Component, Default, Clone)] struct B;
#[derive(Component, Default, Clone)] struct C;

fn one_child() -> impl Scene {
    bsn! { Children [ A B C ] }          // ONE child with A, B and C
}

fn two_children() -> impl Scene {
    bsn! { Children [ A B -- C ] }       // a child with A and B, and one with C
}
```

## Observers live in the scene

`on(...)` attaches an observer to the entity it is written on (see [[Events and observers]]). The click handler sits next to the thing that is clicked:

```rust
use bevy::{prelude::*, ui_widgets::Activate};

#[derive(Resource, Default)]
struct Score(u32);

fn score_button() -> impl Scene {
    bsn! {
        bevy::ui_widgets::Button
        on(|_: On<Activate>, mut score: ResMut<Score>| score.0 += 1)
        Children [ Text("+1") ]
    }
}
```

## A 3D scene, the 0.20 way

This is Bevy's own `3d_scene` example. `bsn_list!` is a list of **root** entities, `asset_value` turns a value into an asset handle, and `.spawn()` makes the function a startup system:

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

Compare with [[Meshes and materials]], which builds the same scene with `commands.spawn`. Here the entities are *named* (`#Cube` adds `Name("Cube")`), nothing needs `Assets<Mesh>` in a system, and the whole world is one function.

## Where BSN shines: UI

UI is a tree of entities, so BSN fits it best: see [[UI layout]] and [[Buttons and widgets]].

## Next

[[BSN syntax]] lists every piece of notation on one page.
