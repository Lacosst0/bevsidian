---
parent: 2D
icon: sliders-horizontal
tags: [2d]
---
# 2D lighting crates

Bevy has no 2D lights of its own. One crate keeps it simple, the other goes for the full look.

| | [[bevy_light_2d]] | [[bevy_firefly]] |
|:--|:--|:--|
| Bevy | #0.20 | #0.19 |
| Lights | point and spot | point, spot, with falloff and banding |
| Shadows | hard, from occluders | soft |
| Normal maps | no | yes |
| Setup | one plugin, a few components | a plugin plus per-sprite normal maps |

> [!TIP]
> - Moody dungeons today: [[bevy_light_2d]].
> - Lit pixel art with normal maps, Godot-style: [[bevy_firefly]].
> - The idea is the same in both: lights, plus occluders that cast shadows. Switching later is cheap.

## Next

[[2D]], [[Make it look good]].
