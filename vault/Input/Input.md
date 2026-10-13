---
icon: keyboard
tags: [input, beginner]
---
# Input

Read what the player does and turn it into game actions. #input #beginner

## Three layers

1. **Raw state**: "is Space down right now?" ([[Keyboard and mouse]], [[Gamepad]], [[Touch]]). Easy, and enough for most games.
2. **Events on things**: "the player clicked *this entity*" ([[Picking]]). Great for UI, menus, board games, strategy.
3. **Actions**: "Jump", "Fire", mapped to many devices and rebindable ([[Input mapping]]). The way to ship a game.

## A taste

```rust
use bevy::prelude::*;

fn jump(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>) {
    if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
        info!("jump!");
    }
}
```

## The notes

| Note | You learn |
|:--|:--|
| [[Keyboard and mouse]] | pressed, just pressed, axes, mouse position and motion |
| [[Gamepad]] | buttons, sticks, rumble |
| [[Touch]] | taps and drags on phones |
| [[Picking]] | clicks and drags on entities |
| [[Input mapping]] | actions, rebinding, plus the best crates |

## Libraries

[[bevy_enhanced_input]] (contexts and actions, observer-based), [[leafwing-input-manager]] (action maps), and [[bevy_egui]] for debug UI input.
