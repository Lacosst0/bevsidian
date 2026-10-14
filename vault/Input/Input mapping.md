---
parent: Keyboard and mouse
icon: sliders-horizontal
tags: [input]
---
# Input mapping

Games do not have "Space pressed". They have **Jump**. Mapping keys to actions lets players rebind, use a gamepad, and lets you reuse code. #input

## By hand: a resource of actions

You can do it yourself in a few lines. Read devices in one system, write the game's own intent to a resource, and let every other system read only that.

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
struct PlayerInput {
    movement: Vec2,
    jump: bool,
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut input: ResMut<PlayerInput>,
) {
    let mut movement = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) { movement.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { movement.x += 1.0; }
    let mut jump = keys.just_pressed(KeyCode::Space);

    for gamepad in &gamepads {
        movement += gamepad.left_stick();
        jump |= gamepad.just_pressed(GamepadButton::South);
    }

    input.movement = movement.clamp_length_max(1.0);
    input.jump = jump;
}

fn player_movement(input: Res<PlayerInput>, mut player: Single<&mut Transform, With<Camera2d>>) {
    player.translation += input.movement.extend(0.0);
}

fn main() {
    App::new()
        .init_resource::<PlayerInput>()
        .add_systems(Update, (read_input, player_movement).chain())
        .run();
}
```

Enough for a jam game. For rebinding screens, several players, input *contexts* (on foot vs in a car) and analog processing, use a crate.

## The two good crates

| Crate | Style | Choose it when |
|:--|:--|:--|
| [[bevy_enhanced_input]] | **contexts, actions, bindings** as components; actions trigger **observer events** (`Start`, `Fire`, `Complete`...) | you want modern Bevy style, per-entity input, modifiers (dead zones, scaling) and conditions (hold, tap, chords) |
| [[leafwing-input-manager]] | an `ActionState<Action>` + `InputMap<Action>` per entity, where your own enum names the actions | you like polling `action_state.pressed(&Action::Jump)` and want a long track record |

Both support keyboard, mouse, gamepad and rebinding. The status column in [[Input]] says which supports Bevy 0.20 today.

## Design tips

- **Name actions after intent** (`Jump`, `Interact`), not devices.
- **Read input in `Update`**, apply it in `FixedUpdate` if you simulate physics there ([[Time and timers]]).
- **Keep a dead zone** on sticks and a "buffer" of a few frames for jump presses: it feels much better.
- **Let players rebind.** Store the map in a file ([[Saving and loading]]).

## Next

You can now build game feel. Hear it: [[Audio]]. Move it physically: [[Physics]].
