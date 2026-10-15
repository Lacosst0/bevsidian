---
parent: UI
icon: sliders-horizontal
tags: [ui]
---
# UI crates

Ways to build UI beyond hand-written Bevy UI nodes. They sit at very different levels, so compare by what you want to write.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_hui]] | pseudo-HTML templates, reloaded while the game runs | #0.20 | you want to write UI like web pages |
| [[bevy_lunex]] | a layout engine, with world-space UI in 2D and 3D | #0.20 (git) | you need in-world screens or any-aspect layouts |
| [[bevy_egui]] | immediate-mode UI for tools | #0.20 | debug panels and editors, not the game's own UI |
| [[bevy_radioactive]] | reactive BSN scenes: signals, `watch`, `view!` | #0.20 (git) | UI that changes with game data |

## Which one?

- Game UI described in files: [[bevy_hui]].
- UI that lives in the 3D world: [[bevy_lunex]].
- Tools and inspectors: [[bevy_egui]].
- UI that reacts to data in plain Bevy UI: [[bevy_radioactive]].

## Next

[[UI]], [[UI layout]].
