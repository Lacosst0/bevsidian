---
parent: 2D
icon: sliders-horizontal
tags: [2d]
---
# Tilemap crates

Two crates for big tile worlds. They are not rivals: the second is built on the first.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_ecs_tilemap]] | a tilemap engine: square, isometric and hex grids, many layers, animated tiles | #0.19 | you want full control of the tiles |
| [[bevy_ecs_ldtk]] | loads levels drawn in the LDtk editor | #0.19 | level designers work in LDtk |

## Which one?

- A small or simple map: Bevy has a basic built-in `TilemapChunk`, see [[Tilemaps]]. No crate needed.
- Hex or isometric grids, many layers, animated tiles, custom tile data: [[bevy_ecs_tilemap]].
- Levels designed in an editor, with entities and fields placed by a designer: [[bevy_ecs_ldtk]]. It uses `bevy_ecs_tilemap` underneath, so you get both.

## Next

[[Tilemaps]] for the built-in way, [[2D]].
