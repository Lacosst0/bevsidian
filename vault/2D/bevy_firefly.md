---
lib: true
parent: 2D lighting crates
icon: package
tags: [2d]
check: skip
---
# bevy_firefly

**2D lighting** in the style of Unity and Godot: soft shadows, normal maps, light banding, z-sorting with lights, occluders from shapes. A more featureful alternative to [[bevy_light_2d]]. #2d

#0.19

```toml
bevy_firefly = "0.19"
```

## Why consider it

- Normal-mapped sprites that react to lights (crates, walls look 3D).
- Soft and hard shadows from occluder shapes.
- Light banding for a retro look.
- Aims to track every new Bevy release.

See the `flashlight`, `shapes` and `stress` examples in the repository.

## Choosing

| | bevy_light_2d | bevy_firefly |
|:--|:--|:--|
| Bevy | #0.20 | #0.19 |
| Features | lights, shadows | + normal maps, banding, z-sorting |

## Links

[crates.io](https://crates.io/crates/bevy_firefly) · [docs.rs](https://docs.rs/bevy_firefly) · [repo](https://github.com/PVDoriginal/firefly)

## Next

[[bevy_light_2d]], [[2D]].
