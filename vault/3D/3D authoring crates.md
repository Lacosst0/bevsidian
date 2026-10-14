---
parent: 3D
icon: sliders-horizontal
tags: [3d]
---
# 3D authoring crates

Make 3D levels in an editor instead of code. The two crates differ by editor.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_trenchbroom]] | Quake-style levels from TrenchBroom: brushes become meshes and colliders | pending `0.14` | you like building rooms from brushes |
| [[bevy_skein]] | Blender: attach Bevy components to objects, export glTF | released `0.7.0-rc.1` (pin it) | your artists work in Blender |

## Which one?

- Blender is your tool: [[bevy_skein]]. Components are set in Blender and inserted when the scene spawns.
- You build levels like a Quake map: [[bevy_trenchbroom]].
- Plain models with no custom data: no crate, see [[Models and glTF]].

## Next

[[Models and glTF]], [[3D]].
