---
lib: true
parent: 3D authoring crates
icon: package
tags: [3d]
crates: bevy_skein=0.7.0-rc.1
---
# bevy_skein

**Blender → Bevy components.** Attach your reflected Bevy components to objects, meshes or materials *inside Blender*, export glTF, and they are inserted when the scene spawns. Level designers can place a "Door" with `locked: true` without code. #3d

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `0.7.0-rc.1` targets **Bevy 0.20** (a release candidate, so pin it explicitly).

```toml
bevy_skein = "0.7.0-rc.1"
```

Plus the **Skein extension for Blender** (see the project's docs).

## Reflect your components

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

## Workflow

1. Run the game once so Skein can write the type registry (your components appear in Blender's panel).
2. In Blender, select an object and add `Door`; fill in its fields.
3. Export glTF.
4. Spawn it like any model ([[Models and glTF]]): the entity has a `Door` component.

## Alternatives

- [[bevy_trenchbroom]]: Quake-style brush level editing with entities and properties.
- By hand: spawn the model, then add components from node [`Name`]s in an observer ([[Models and glTF]]).

## Links

[crates.io](https://crates.io/crates/bevy_skein) · [docs.rs](https://docs.rs/bevy_skein) · [repo](https://github.com/rust-adventure/skein)

## Next

[[Models and glTF]].
