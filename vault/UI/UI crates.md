---
parent: UI
icon: sliders-horizontal
tags: [ui]
---
# UI crates

Ways to build UI beyond hand-written Bevy UI nodes. They work at very different levels, so compare by what you want to write.

| | [[bevy_radioactive]] | [[bevy_hui]] | [[bevy_lunex]] | [[bevy_egui]] |
|:--|:--|:--|:--|:--|
| Bevy | #0.20 (git, 0.6) | #0.20 | #0.20 (git) | #0.20 |
| You write | BSN scenes that declare what they react to | HTML-like templates | layouts in code | immediate-mode Rust |
| Updates | reactive: changes follow your data | hot reload of templates | each frame | each frame |
| Best for | game UI that follows game data | designers who like markup | in-world screens, any aspect ratio | debug panels and editors |

> [!TIP]
> - Game UI that reacts to data, staying in plain Bevy UI: [[bevy_radioactive]].
> - UI described in files: [[bevy_hui]].
> - UI living in the 3D world: [[bevy_lunex]].
> - Tools and inspectors, not the game's UI: [[bevy_egui]].

## Next

[[UI]], [[UI layout]].
