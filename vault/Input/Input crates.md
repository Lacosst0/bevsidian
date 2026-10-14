---
parent: Input
icon: sliders-horizontal
tags: [input]
---
# Input crates

Two crates for actions and rebinding. Both are ready on their `main` branch for Bevy 0.20; the releases are pending.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_enhanced_input]] | contexts, actions and bindings; observer based | git | you like observers and layered contexts (on foot, in a car) |
| [[leafwing-input-manager]] | an action enum, an `InputMap` and `ActionState` | git | you want the long-established, feature-full option |

## Which one?

- Many control schemes that switch (character, vehicle, menu): [[bevy_enhanced_input]].
- A single action enum with rebinding and gamepad support: [[leafwing-input-manager]].
- Few actions: skip both and use [[Keyboard and mouse]] directly.

## Next

[[Input mapping]], [[Input]].
