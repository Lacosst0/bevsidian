---
icon: package
tags: [lib, 3d]
check: skip
---
# bevy_atmosphere

A **procedural sky** (Rayleigh and Mie scattering) with a moving sun. Historical entry: Bevy now ships its own physically based `Atmosphere` ([[Environment]]), so new projects should use that. #lib #3d

> [!WARNING]
> **Not maintained for current Bevy.** The last release (`0.13`, 2025) targets Bevy 0.16.

## Use instead

On Bevy 0.20: add an `Atmosphere` to the camera and move a `DirectionalLight` ([[Environment]], [[Lighting]]). For stylised skies, a skybox ([[Environment]]) or a sky material ([[Custom materials]]).

## Links

[crates.io](https://crates.io/crates/bevy_atmosphere) · [repo](https://github.com/JonahPlusPlus/bevy_atmosphere)

## Next

[[Environment]], [[Libraries]].
