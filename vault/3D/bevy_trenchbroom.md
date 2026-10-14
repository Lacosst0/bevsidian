---
lib: true
parent: 3D authoring crates
icon: package
tags: [3d]
check: skip
---
# bevy_trenchbroom

Build levels with **TrenchBroom**, the Quake map editor: draw rooms from brushes, place entities with properties, and load the `.map` files in Bevy. Brushes become meshes and colliders; entity classes become your components. #3d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.14` targets **Bevy 0.19**.

```toml
bevy_trenchbroom = "0.14"
```

## Why it is popular for 3D games

- **Fast greyboxing**: sculpt a level in minutes with a tool built for it.
- **Entities with properties**: a "light" with a colour, a "door" with a key; Bevy components defined in Rust become entity classes in the editor (it writes the game config for TrenchBroom).
- **Hot reload** of maps while the game runs ([[Hot reloading]]).
- Textures, lightmaps and collision from your physics crate ([[Avian]] or [[bevy_rapier]]).

## Workflow

1. Add the plugin and define entity classes with a derive.
2. Let the crate write the TrenchBroom game config.
3. Draw the map; save `.map`.
4. Load it as an asset and spawn ([[Assets]], [[Scenes and BSN]]).

Follow the crate's book for the exact setup: the API shifts with each Bevy release.

## Alternatives

[[bevy_skein]] (Blender workflow) or plain glTF levels ([[Models and glTF]]).

## Links

[crates.io](https://crates.io/crates/bevy_trenchbroom) · [docs.rs](https://docs.rs/bevy_trenchbroom) · [repo](https://github.com/Noxmore/bevy_trenchbroom)

## Next

[[3D]].
