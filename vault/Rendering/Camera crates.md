---
parent: Rendering
icon: sliders-horizontal
tags: [rendering]
---
# Camera crates

Ready-made camera controls. Bevy ships its own pan and orbit controllers, so you probably do not need either crate.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_pancam]] | pan and zoom for 2D: drag, scroll to zoom at the cursor, WASD | #0.19 | you are on Bevy 0.19 or need its extras |
| [[bevy_panorbit_camera]] | orbit for 3D: drag to rotate, scroll to zoom, pan | #0.19 | you are on an older Bevy |

## Which one?

- Most projects: use the built-in `PanCamera` and `PanOrbitCamera` controllers, see [[Cameras]].
- Older Bevy: [[bevy_pancam]] for 2D maps and strategy games, [[bevy_panorbit_camera]] for model viewers and editors.

## Next

[[Cameras]].
