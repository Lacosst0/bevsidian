---
icon: package
tags: [lib, 3d, ui]
crates: bevy_rich_text3d=0.8
---
# bevy_rich_text3d

**Text as a mesh**, in 2D or 3D, with an HTML-like markup for colours, sizes and effects. Each glyph is its own quad, so shaders can wobble, type out or rainbow individual letters. Complements Bevy's own text ([[Text 2D]], [[Text and images]]). #lib #3d #ui

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_rich_text3d 0.8` supports **Bevy 0.20**.

```toml
bevy_rich_text3d = "0.8"
```

## Text in the world

```rust
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyle, TextAtlas};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Text3dPlugin { load_system_fonts: true, ..default() },
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Text3d::new("Hello, 3D!"),
        Text3dStyle { size: 64.0, ..default() },
        Mesh3d::default(),                 // the plugin fills in the glyph mesh
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),   // the shared glyph atlas
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
    ));
    let _ = &mut meshes;
}
```

The material uses the crate's glyph atlas as its texture. `Mesh2d` + `ColorMaterial` works the same for 2D.

## Rich text

`Text3d::parse` reads a small markup: `{style:text}` is like `<style>text</style>`, and `{name}` inserts a value from the world (a score, a name):

```text
"A {color:red|danger} zone, {bold|careful}!"
```

See `Text3d::parse` in the docs for the exact tags.

## When to use it

- Floating damage numbers and name tags that must be crisp at any distance.
- Signs and screens inside a 3D scene.
- Per-letter effects: typewriter, shake, rainbow (see the `typewriter`, `rainbow` and `spooky` examples).
- Right-to-left and emoji text (`arabic`, `emoji`).

For menus use [[UI]]; for plain labels over sprites, [[Text 2D]].

## Links

[crates.io](https://crates.io/crates/bevy_rich_text3d) · [docs.rs](https://docs.rs/bevy_rich_text3d) · [repo](https://github.com/mintlu8/bevy_rich_text3d)

## Next

[[3D]], [[Libraries]].
