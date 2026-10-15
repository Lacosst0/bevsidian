---
parent: 3D
icon: sliders-horizontal
tags: [3d]
---
# 3D effect crates

Three unrelated crates for things Bevy does not do on its own. Pick by the effect you want, not against each other.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_mod_outline]] | outlines around meshes (selection glow, borders) | #0.19 | a unit or item should be highlighted |
| [[bevy_water]] | an animated water material with waves and floating objects | #0.19 | you need an ocean or lake |
| [[bevy_rich_text3d]] | text as a mesh in 2D or 3D, with markup and per-letter effects | #0.20 | text lives in the world, or you want animated letters |

## Which one?

- Highlight selection: [[bevy_mod_outline]].
- Water: [[bevy_water]]; check its Bevy version first.
- Text in the world: [[bevy_rich_text3d]]. For HUD text use normal UI text ([[Text and images]]).

## Next

[[Environment]], [[Lighting]].
