---
parent: Rendering
icon: sliders-horizontal
tags: [rendering]
---
# Camera crates

Ready-made camera controls. Bevy 0.20 now ships its own pan and orbit controllers, so on 0.20 you probably do not need either crate.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_pancam]] | pan and zoom for 2D: drag, scroll to zoom at the cursor, WASD | pending `0.21` | you are on Bevy 0.19 or need its extras |
| [[bevy_panorbit_camera]] | orbit for 3D: drag to rotate, scroll to zoom, pan | pending `0.35`; built in now | you are on an older Bevy |

## Which one?

- On Bevy 0.20: use the built-in `PanCamera` and `PanOrbitCamera` controllers, see [[Cameras]].
- Older Bevy: [[bevy_pancam]] for 2D maps and strategy games, [[bevy_panorbit_camera]] for model viewers and editors.

## Next

[[Cameras]].
