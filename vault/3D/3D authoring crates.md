---
icon: package
tags: [3d]
crates: bevy_skein=0.7.0-rc.1
---
# 3D authoring crates

Make 3D levels in an editor instead of code: Quake-style brushes with TrenchBroom, or components set in Blender with Skein. See also [[Models and glTF]].

## bevy_trenchbroom

Build levels with **TrenchBroom**, the Quake map editor: draw rooms from brushes, place entities with properties, and load the `.map` files in Bevy. Brushes become meshes and colliders; entity classes become your components. #3d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.14` targets **Bevy 0.19**.

```toml
bevy_trenchbroom = "0.14"
```

### Why it is popular for 3D games

- **Fast greyboxing**: sculpt a level in minutes with a tool built for it.
- **Entities with properties**: a "light" with a colour, a "door" with a key; Bevy components defined in Rust become entity classes in the editor (it writes the game config for TrenchBroom).
- **Hot reload** of maps while the game runs ([[Hot reloading]]).
- Textures, lightmaps and collision from your physics crate ([[Avian]] or [[bevy_rapier]]).

### Workflow

1. Add the plugin and define entity classes with a derive.
2. Let the crate write the TrenchBroom game config.
3. Draw the map; save `.map`.
4. Load it as an asset and spawn ([[Assets]], [[Scenes and BSN]]).

Follow the crate's book for the exact setup: the API shifts with each Bevy release.

### Alternatives

[[3D authoring crates#bevy_skein|bevy_skein]] (Blender workflow) or plain glTF levels ([[Models and glTF]]).

### Links

[crates.io](https://crates.io/crates/bevy_trenchbroom) · [docs.rs](https://docs.rs/bevy_trenchbroom) · [repo](https://github.com/Noxmore/bevy_trenchbroom)

## bevy_skein

**Blender → Bevy components.** Attach your reflected Bevy components to objects, meshes or materials *inside Blender*, export glTF, and they are inserted when the scene spawns. Level designers can place a "Door" with `locked: true` without code. #3d

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `0.7.0-rc.1` targets **Bevy 0.20** (a release candidate, so pin it explicitly).

```toml
bevy_skein = "0.7.0-rc.1"
```

Plus the **Skein extension for Blender** (see the project's docs).

### Reflect your components

```rust
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SkeinPlugin::default()))
        .run();
}

// `reflect(Component)` is what makes it visible to Blender.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Door {
    locked: bool,
    key_name: String,
}
```

### Workflow

1. Run the game once so Skein can write the type registry (your components appear in Blender's panel).
2. In Blender, select an object and add `Door`; fill in its fields.
3. Export glTF.
4. Spawn it like any model ([[Models and glTF]]): the entity has a `Door` component.

### Alternatives

- [[3D authoring crates#bevy_trenchbroom|bevy_trenchbroom]]: Quake-style brush level editing with entities and properties.
- By hand: spawn the model, then add components from node [`Name`]s in an observer ([[Models and glTF]]).

### Links

[crates.io](https://crates.io/crates/bevy_skein) · [docs.rs](https://docs.rs/bevy_skein) · [repo](https://github.com/rust-adventure/skein)
