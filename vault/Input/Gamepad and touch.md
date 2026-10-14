---
icon: gamepad-2
tags: [input]
---
# Gamepad and touch

Controllers and fingers. For keyboard and mouse see [[Keyboard and mouse]]; for actions on top of any device see [[Input mapping]].

## Gamepad

Each connected controller is an **entity** with a [`Gamepad`] component. #input

### Buttons and sticks

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

### Dead zones

Sticks rarely rest exactly at zero. Bevy applies a default dead zone; tune it per axis with `GamepadSettings`, or ignore tiny values yourself as above.

### Connections

A controller appearing or disappearing is an entity being spawned or despawned. React with observers ([[Component lifecycle]]):

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_observer(|add: On<Add<Gamepad>>| info!("controller connected: {}", add.entity))
        .add_observer(|remove: On<Remove<Gamepad>>| info!("controller gone: {}", remove.entity));
}
```

### Rumble

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

### Several players

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

## Touch

Phones and tablets give you fingers instead of a mouse. #input

### Touches resource

```rust
use bevy::prelude::*;

fn touches(touches: Res<Touches>) {
    for finger in touches.iter() {
        info!("finger {} at {} (moved {})", finger.id(), finger.position(), finger.delta());
    }
    for finger in touches.iter_just_pressed() {
        info!("tap started at {}", finger.position());
    }
    for finger in touches.iter_just_released() {
        info!("finger {} lifted", finger.id());
    }
}
```

Positions are in window pixels from the top-left, like `cursor_position`. Convert to the world with `camera.viewport_to_world_2d` ([[Cameras]]).

### Pointer events work for fingers too

Picking ([[Picking]]) treats touch like a mouse, so `PointerClick` and `PointerDrag` on your entities work on phones and desktops with no extra code. Prefer them for buttons and draggable things.

### Gestures

Pinch to zoom and rotate arrive as messages:

```rust
use bevy::{input::gestures::PinchGesture, prelude::*};

fn pinch(mut gestures: MessageReader<PinchGesture>, mut projection: Single<&mut Projection, With<Camera2d>>) {
    for gesture in gestures.read() {
        if let Projection::Orthographic(ortho) = &mut **projection {
            ortho.scale *= 1.0 - gesture.0;
        }
    }
}
```

(Gesture messages are currently produced on macOS; on other platforms track two fingers from `Touches` yourself.)

### Virtual sticks

No gamepad on a phone: draw a joystick with UI ([[UI layout]]) and read the drag with pointer events:

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
struct VirtualStick(Vec2);   // -1..1 on each axis

fn on_drag(drag: On<PointerDrag>, mut sticks: Query<&mut VirtualStick>) {
    if let Ok(mut stick) = sticks.get_mut(drag.entity) {
        stick.0 = (drag.distance / 60.0).clamp_length_max(1.0) * Vec2::new(1.0, -1.0);
    }
}

fn on_release(end: On<PointerDragEnd>, mut sticks: Query<&mut VirtualStick>) {
    if let Ok(mut stick) = sticks.get_mut(end.entity) {
        stick.0 = Vec2::ZERO;
    }
}
```

### Design for fingers

- Make touch targets at least 44 px.
- Do not rely on hover.
- Test on a real device; the browser build ([[Web build]]) works on phones.
