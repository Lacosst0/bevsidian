---
icon: mouse-pointer
tags: [input]
---
# Keyboard and mouse

Input lives in resources. Read them in a system that runs every frame. #input

## Keys

[`ButtonInput<KeyCode>`] remembers the state of every key:

| Method | True when |
|:--|:--|
| `pressed(key)` | the key is held (every frame) |
| `just_pressed(key)` | it went down this frame (once) |
| `just_released(key)` | it came up this frame (once) |

```rust
use bevy::prelude::*;

fn controls(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        info!("jump once");
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        info!("sprinting while held");
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        info!("going left");
    }
}
```

`KeyCode` names *physical* keys (`KeyW` is the key in the W position on a US layout, so WASD works on every layout). For typed characters use text events instead.

## Move a player

A direction vector from the keys, normalised so diagonals are not faster:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player;

fn walk(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }

    const SPEED: f32 = 200.0;
    player.translation += (direction.normalize_or_zero() * SPEED * time.delta_secs()).extend(0.0);
}
```

Always `normalize_or_zero`, and always multiply by `time.delta_secs()` ([[Time and timers]]).

> [!TIP]
> The previews on this site can't read your keyboard, so click-driven versions are shown instead: click the stage below and the ship flies to the click.

## Mouse buttons and position

```rust
use bevy::prelude::*;

fn mouse(buttons: Res<ButtonInput<MouseButton>>, window: Single<&Window>) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = window.cursor_position() {
            info!("clicked at {position} (pixels from the top-left)");
        }
    }
}
```

To get a **world** position in 2D use `camera.viewport_to_world_2d` ([[Cameras]]).

## Mouse motion and scroll

Motion and wheel come as [[Messages|messages]] or as accumulated resources:

```rust
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};

fn look(motion: Res<AccumulatedMouseMotion>, scroll: Res<AccumulatedMouseScroll>) {
    if motion.delta != Vec2::ZERO {
        info!("mouse moved by {}", motion.delta);   // use for first-person looking
    }
    if scroll.delta.y != 0.0 {
        info!("wheel {}", scroll.delta.y);
    }
}
```

Grab the cursor for first-person games: [[Windows]].

## Click to move: a demo

Click anywhere on the stage. The ship sets a `Target`, a system moves it there at a fixed speed and a second one turns it to face its target. This is the structure of most "move to point" mechanics.

```rust example=movement
use bevy::prelude::*;

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Marker;

/// Where the ship is going.
#[derive(Component)]
struct Target(Vec2);

#[derive(Component)]
struct Sky;

fn spawn_ship(mut commands: Commands) {
    commands.spawn((
        Ship,
        Sprite::from_color(Color::srgb(0.35, 0.75, 0.9), Vec2::new(28.0, 12.0)),
        Transform::from_xyz(-100.0, 0.0, 1.0),
    ));
    commands.spawn((
        Marker,
        Sprite::from_color(Color::WHITE, Vec2::new(6.0, 6.0)),
        Transform::from_xyz(0.0, 0.0, 0.5),
    ));
}

fn set_target(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    ship: Query<Entity, With<Ship>>,
    mut marker: Query<&mut Transform, With<Marker>>,
    mut commands: Commands,
) {
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    for ship in &ship {
        commands.entity(ship).insert(Target(at.truncate()));
    }
    for mut marker in &mut marker {
        marker.translation = at.truncate().extend(0.5);
    }
}

fn fly(time: Res<Time>, mut ships: Query<(&mut Transform, &Target)>) {
    const SPEED: f32 = 90.0;
    for (mut transform, target) in &mut ships {
        let to_target = target.0 - transform.translation.truncate();
        // Never overshoot: move at most the distance that is left.
        let step = (SPEED * time.delta_secs()).min(to_target.length());
        transform.translation += (to_target.normalize_or_zero() * step).extend(0.0);
    }
}

fn turn(mut ships: Query<(&mut Transform, &Target)>) {
    for (mut transform, target) in &mut ships {
        let to_target = target.0 - transform.translation.truncate();
        if to_target.length() > 1.0 {
            transform.rotation = Quat::from_rotation_z(to_target.to_angle());
        }
    }
}
```

## Text typed, not keys

For typing characters (chat, names) use [[Text and images#Text input|Text input]] or the `KeyboardInput` message's `logical_key`.

## Next

[[Gamepad and touch#Gamepad|Gamepad]]. Or [[Picking]] for clicking on things.
