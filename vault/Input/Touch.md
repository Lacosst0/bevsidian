---
icon: smartphone
tags: [input]
---
# Touch

Phones and tablets give you fingers instead of a mouse. #input

## Touches resource

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

## Pointer events work for fingers too

Picking ([[Picking]]) treats touch like a mouse, so `PointerClick` and `PointerDrag` on your entities work on phones and desktops with no extra code. Prefer them for buttons and draggable things.

## Gestures

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

## Virtual sticks

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

## Design for fingers

- Make touch targets at least 44 px.
- Do not rely on hover.
- Test on a real device; the browser build ([[Web build]]) works on phones.

## Next

[[Picking]].
