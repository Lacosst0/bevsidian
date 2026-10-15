---
parent: Input
icon: sliders-horizontal
tags: [input]
---
# Input crates

Two crates for actions and rebinding. Both build on #0.20 from their git `main` branch.

| | [[bevy_enhanced_input]] | [[leafwing-input-manager]] |
|:--|:--|:--|
| Bevy | #0.20 (git) | #0.20 (git) |
| Model | contexts with actions, read by observers | an action enum, an `InputMap`, an `ActionState` |
| Switching schemes | layered contexts (on foot, driving, menu) | swap or merge maps yourself |
| Feels like | Unreal's Enhanced Input | the long-standing Bevy default |

> [!TIP]
> - Control schemes that switch often: [[bevy_enhanced_input]].
> - One action enum, rebinding and gamepads: [[leafwing-input-manager]].
> - A handful of keys: skip both and read input directly ([[Input]]).

## Next

[[Input mapping]], [[Input]].
