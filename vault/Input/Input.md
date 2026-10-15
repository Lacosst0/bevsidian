---
icon: keyboard
tags: [input]
---
# Input

Read what the player does and turn it into game actions. #input

## Three layers

1. **Raw state**: "is Space down right now?" ([[Keyboard and mouse]], [[Gamepad and touch#Gamepad|Gamepad]], [[Gamepad and touch#Touch|Touch]]). Easy, and enough for most games.
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
| [[Gamepad and touch#Gamepad\|Gamepad]] | buttons, sticks, rumble |
| [[Gamepad and touch#Touch\|Touch]] | taps and drags on phones |
| [[Picking]] | clicks and drags on entities |
| [[Input mapping]] | actions, rebinding, plus the best crates |

## Libraries

Comparisons of crates that do similar jobs: [[Input crates]].

| Crate | For | Bevy |
|:--|:--|:--|
| [[bevy_enhanced_input]] | actions, contexts, modifiers; **observer based** | #0.20 (git) |
| [[leafwing-input-manager]] | action enum + `InputMap`, remapping | #0.20 (git) |
| `bevy_mod_picking` | old picking crate | built in built into Bevy ([[Picking]]) |
