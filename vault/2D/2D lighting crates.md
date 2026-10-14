---
parent: 2D
icon: sliders-horizontal
tags: [2d]
---
# 2D lighting crates

Bevy has no 2D lights of its own. Two crates add them, one simple and one rich.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_light_2d]] | point and spot lights with shadows; walls block light | released `0.11` | you want moody dungeons now, on Bevy 0.20 |
| [[bevy_firefly]] | Unity/Godot-style lighting: soft shadows, normal maps, light banding | pending `0.19` | you need normal maps or the richer look, and can wait or stay on 0.19 |

## Which one?

- On Bevy 0.20 today: [[bevy_light_2d]].
- Normal-mapped sprites, banded or stylised lights: [[bevy_firefly]], once it supports your Bevy version.
- Only the two crates differ in setup; the idea (lights plus occluders) is the same.

## Next

[[2D]], [[Make it look good]].
