---
parent: 2D
icon: sliders-horizontal
tags: [2d]
---
# Animation crates

Two crates that animate for you. They solve different problems and work well together.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_tweening]] | animates any value (position, scale, colour, UI size) with easing, loops and sequences | pending `0.16` | something should move, grow, fade or shake |
| [[bevy_spritesheet_animation]] | sprite-sheet clips: define frames, speed and events | pending `7.0.1` | a character cycles through frames of a sprite sheet |

## Which one?

- Frames of a character (walk, idle, attack): the built-in way is in [[Sprite animation]]; [[bevy_spritesheet_animation]] saves you writing the timer system.
- Smooth motion between two values: [[bevy_tweening]]. It also does UI and 3D.
- Both: a tween moves the sprite while a clip plays its frames.

## Next

[[Sprite animation]], [[Game feel]] for what to tween.
