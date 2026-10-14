---
icon: keyboard
tags: [input]
---
# Input

Read what the player does and turn it into game actions. #input

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

Status for **Bevy 0.20**, checked 2026-10-13: **released** (a crates.io release uses 0.20) · **git** (the `main` branch is on 0.20; use a git dependency) · **pending** (the latest release targets an older Bevy) · **built in** (Bevy has it now). Check crates.io before you start.

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_enhanced_input]] | actions, contexts, modifiers; **observer based** | git |
| [[leafwing-input-manager]] | action enum + `InputMap`, remapping | git |
| `bevy_mod_picking` | old picking crate | built in built into Bevy ([[Picking]]) |
