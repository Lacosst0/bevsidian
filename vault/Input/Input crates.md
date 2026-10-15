---
parent: Input
icon: sliders-horizontal
tags: [input]
---
# Input crates

Two crates for actions and rebinding. Both work with #0.20 from their git `main` branch.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_enhanced_input]] | contexts, actions and bindings; observer based | #0.20 (git) | you like observers and layered contexts (on foot, in a car) |
| [[leafwing-input-manager]] | an action enum, an `InputMap` and `ActionState` | #0.20 (git) | you want the long-established, feature-full option |

## Which one?

- Many control schemes that switch (character, vehicle, menu): [[bevy_enhanced_input]].
- A single action enum with rebinding and gamepad support: [[leafwing-input-manager]].
- Few actions: skip both and read the keyboard and mouse directly ([[Input#Keyboard and mouse|Input]]).

## Next

[[Input mapping]], [[Input]].
