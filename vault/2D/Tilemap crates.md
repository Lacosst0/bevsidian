---
parent: 2D
icon: sliders-horizontal
tags: [2d]
---
# Tilemap crates

Two crates for big tile worlds. They aren't rivals: the second is built on the first.

| | [[bevy_ecs_tilemap]] | [[bevy_ecs_ldtk]] |
|:--|:--|:--|
| Bevy | #0.19 | #0.19 |
| What it is | a tilemap engine: every tile an entity | levels from the LDtk editor |
| Grids | square, isometric, hexagonal | whatever LDtk draws |
| Layers, animated tiles | yes | yes (through `bevy_ecs_tilemap`) |
| Who makes the levels | your code | a designer, in LDtk |

> [!TIP]
> - A small map, one layer: no crate. Bevy's `TilemapChunk` does it ([[Tilemaps]]).
> - Hex or isometric grids, many layers, tiles with behaviour: [[bevy_ecs_tilemap]].
> - Levels drawn in an editor, with entities and fields placed by hand: [[bevy_ecs_ldtk]]. You get `bevy_ecs_tilemap` underneath for free.

## Next

[[Tilemaps]], [[2D]].
