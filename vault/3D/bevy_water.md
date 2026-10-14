---
lib: true
parent: 3D effect crates
icon: package
tags: [3d]
check: skip
---
# bevy_water

A **dynamic water material with waves** for 3D: a plane that animates, with depth colouring, foam and objects that float with the waves (the `pirates` example). #3d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.18.1` targets **Bevy 0.18**; expect to wait for an update.

```toml
bevy_water = "0.18"
```

## What it gives you

- A water material built on `StandardMaterial` ([[Custom materials]] shows how such an extension works).
- A **wave height function** you can call on the CPU, so boats and buoys bob at the right height.
- Works on WebGPU and WebGL2 builds ([[Web build]]).

## No water crate?

A simple version: a plane with a custom shader that offsets vertices by `sin(position * frequency + time)` and tints by depth. Start from [[Custom materials]].

## Links

[crates.io](https://crates.io/crates/bevy_water) · [docs.rs](https://docs.rs/bevy_water) · [repo](https://github.com/Neopallium/bevy_water)

## Next

[[Environment]].
