---
parent: 3D
icon: sliders-horizontal
tags: [3d]
---
# 3D authoring crates

Make 3D levels in an editor instead of code. The two crates differ by editor.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_trenchbroom]] | Quake-style levels from TrenchBroom: brushes become meshes and colliders | #0.19 | you like building rooms from brushes |
| [[bevy_skein]] | Blender: attach Bevy components to objects, export glTF | #0.20 (pin it) | your artists work in Blender |

## Which one?

- Blender is your tool: [[bevy_skein]]. Components are set in Blender and inserted when the scene spawns.
- You build levels like a Quake map: [[bevy_trenchbroom]].
- Plain models with no custom data: no crate, see [[Models and glTF]].

## Next

[[Models and glTF]], [[3D]].
