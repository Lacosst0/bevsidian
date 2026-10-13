---
icon: gamepad-2
tags: [input, intermediate]
---
# Gamepad

Each connected controller is an **entity** with a [`Gamepad`] component. #input #intermediate

## Buttons and sticks

```rust
use bevy::prelude::*;

fn gamepad(gamepads: Query<(Entity, &Gamepad)>) {
    for (entity, gamepad) in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {      // A / Cross
            info!("{entity}: jump");
        }
        let left = gamepad.left_stick();                      // Vec2, -1..1
        if left.length() > 0.1 {
            info!("{entity}: moving {left}");
        }
        let trigger = gamepad.get(GamepadButton::RightTrigger2).unwrap_or(0.0);
        if trigger > 0.1 {
            info!("{entity}: firing {trigger}");
        }
    }
}
```

Button names follow position, not label: `South` is the bottom face button (A on Xbox, Cross on PlayStation), `East` is B/Circle, `North` is Y/Triangle.

## Dead zones

Sticks rarely rest exactly at zero. Bevy applies a default dead zone; tune it per axis with `GamepadSettings`, or ignore tiny values yourself as above.

## Connections

A controller appearing or disappearing is an entity being spawned or despawned. React with observers ([[Component lifecycle]]):

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_observer(|add: On<Add<Gamepad>>| info!("controller connected: {}", add.entity))
        .add_observer(|remove: On<Remove<Gamepad>>| info!("controller gone: {}", remove.entity));
}
```

## Rumble

```rust
use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;
use std::time::Duration;

fn rumble(mut requests: MessageWriter<GamepadRumbleRequest>, gamepads: Query<Entity, With<Gamepad>>) {
    for gamepad in &gamepads {
        requests.write(GamepadRumbleRequest::Add {
            gamepad,
            duration: Duration::from_millis(200),
            intensity: GamepadRumbleIntensity::strong_motor(0.8),
        });
    }
}
```

## Several players

Each player is one `Gamepad` entity. Store the entity in your `Player` component, and read *that* gamepad:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player { gamepad: Entity }

fn move_players(players: Query<(&Player, &mut Transform)>, gamepads: Query<&Gamepad>) {
    for (player, mut transform) in players {
        if let Ok(gamepad) = gamepads.get(player.gamepad) {
            transform.translation += gamepad.left_stick().extend(0.0);
        }
    }
}
```

## Next

For one set of "actions" across keyboard and gamepad: [[Input mapping]].
