---
icon: package
tags: [3d, ui]
crates: bevy_rich_text3d=0.8
---
# 3D effect crates

Three small crates for things Bevy does not do alone: outlines, water and text in the 3D world. See also [[Environment]] and [[Lighting]].

## bevy_mod_outline

**Outlines around meshes**: the glowing edge on a selected unit, the border on an interactable. Uses vertex extrusion or a jump-flood algorithm; supports opaque and transparent outlines, render layers, skinned meshes. #3d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.13` targets **Bevy 0.19**.

```toml
bevy_mod_outline = "0.13"
```

### Idea

Add the plugin, then add an outline component to any mesh entity. Toggle it on hover or selection ([[Picking]]):

```rust ignore
// Sketch, see the crate's `shapes` example for the exact fields of your version.
app.add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin::default()));

commands.spawn((
    Mesh3d(mesh),
    MeshMaterial3d(material),
    OutlineVolume { visible: true, width: 4.0, colour: Color::WHITE },
));
```

Hook `PointerOver` / `PointerOut` observers to flip `visible` and you have hover highlights for 3D objects.

### Alternatives

- A second slightly-scaled, back-face-only mesh (cheap and DIY).
- Post-process edge detection ([[Post-processing]]).

### Links

[crates.io](https://crates.io/crates/bevy_mod_outline) · [docs.rs](https://docs.rs/bevy_mod_outline) · [repo](https://github.com/komadori/bevy_mod_outline)

## bevy_water

A **dynamic water material with waves** for 3D: a plane that animates, with depth colouring, foam and objects that float with the waves (the `pirates` example). #3d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.18.1` targets **Bevy 0.18**; expect to wait for an update.

```toml
bevy_water = "0.18"
```

### What it gives you

- A water material built on `StandardMaterial` ([[Custom materials]] shows how such an extension works).
- A **wave height function** you can call on the CPU, so boats and buoys bob at the right height.
- Works on WebGPU and WebGL2 builds ([[Web build]]).

### No water crate?

A simple version: a plane with a custom shader that offsets vertices by `sin(position * frequency + time)` and tints by depth. Start from [[Custom materials]].

### Links

[crates.io](https://crates.io/crates/bevy_water) · [docs.rs](https://docs.rs/bevy_water) · [repo](https://github.com/Neopallium/bevy_water)

## bevy_rich_text3d

**Text as a mesh**, in 2D or 3D, with an HTML-like markup for colours, sizes and effects. Each glyph is its own quad, so shaders can wobble, type out or rainbow individual letters. Complements Bevy's own text ([[Text 2D]], [[Text and images]]). #3d #ui

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_rich_text3d 0.8` supports **Bevy 0.20**.

```toml
bevy_rich_text3d = "0.8"
```

### Text in the world

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

### Rich text

`Text3d::parse` reads a small markup: `{style:text}` is like `<style>text</style>`, and `{name}` inserts a value from the world (a score, a name):

```text
"A {color:red|danger} zone, {bold|careful}!"
```

See `Text3d::parse` in the docs for the exact tags.

### When to use it

- Floating damage numbers and name tags that must be crisp at any distance.
- Signs and screens inside a 3D scene.
- Per-letter effects: typewriter, shake, rainbow (see the `typewriter`, `rainbow` and `spooky` examples).
- Right-to-left and emoji text (`arabic`, `emoji`).

For menus use [[UI]]; for plain labels over sprites, [[Text 2D]].

### Links

[crates.io](https://crates.io/crates/bevy_rich_text3d) · [docs.rs](https://docs.rs/bevy_rich_text3d) · [repo](https://github.com/mintlu8/bevy_rich_text3d)
