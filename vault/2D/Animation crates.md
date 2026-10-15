---
parent: 2D
icon: sliders-horizontal
tags: [2d]
---
# Animation crates

Two crates that animate for you. They solve different problems and work well together.

| | [[bevy_tweening]] | [[bevy_spritesheet_animation]] |
|:--|:--|:--|
| Bevy | #0.19 | #0.19 |
| Animates | any value: position, scale, colour, UI size | frames of a sprite sheet |
| Timing | easing, loops, sequences | clips with speed and events |
| Also does | UI and 3D | 3D sprites |

> [!TIP]
> - Something should move, grow, fade or shake: [[bevy_tweening]].
> - A character cycles walk, idle and attack frames: [[bevy_spritesheet_animation]], or ten lines of your own ([[Sprite animation]]).
> - Both at once is normal: a tween moves the sprite while a clip plays its frames.

## Next

[[Sprite animation]], [[Game feel]] for what to tween.
