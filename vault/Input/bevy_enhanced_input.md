---
lib: true
parent: Input crates
icon: package
tags: [input]
crates: bevy_enhanced_input={ git = "https://github.com/simgine/bevy_enhanced_input" }
---
# bevy_enhanced_input

An **observer-based input manager**: you describe *contexts* (on foot, in a car) with *actions* (Move, Jump) and *bindings* (keys, sticks); your game just observes the actions. Inspired by Unreal's Enhanced Input. #input

> [!IMPORTANT]
> **Bevy 0.20 status** (checked 2026-10-13): the crates.io release `0.26` targets Bevy 0.19. The `main` branch already targets **0.20**: depend on it with `git` until the next release.

```toml
[dependencies]
bevy = "0.20"
bevy_enhanced_input = { git = "https://github.com/simgine/bevy_enhanced_input" }
```

## Why choose it

- Input is **data on entities**, so every player or vehicle has its own context ([[Relationships and hierarchy|entities with children]]).
- **Modifiers** shape raw input: dead zones, smoothing, scaling, negating, delta-time scaling.
- **Conditions** decide when an action fires: pressed, held for 0.5 s, tapped, chorded with another key.
- Actions arrive as [[Events and observers|observer events]] (`Start`, `Fire`, `Complete`...), no polling.
- Rebinding screens are easy: bindings are components you can change.

## The four ideas

| Idea | In code |
|:--|:--|
| **Action**: what the player wants | a type with `#[derive(InputAction)]` and an output type (`bool`, `f32`, `Vec2`) |
| **Context**: a set of actions that are active together | a component registered with `add_input_context::<T>()` |
| **Binding**: a key/stick/button for an action | `bindings![KeyCode::Space]`, `Cardinal::wasd_keys()`, `Axial::left_stick()` |
| **Modifier / condition**: shape or gate the input | components on the action: `Scale`, `DeadZone`, `DeltaScale`... |

## A player that moves and jumps

```rust
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()     // contexts must be registered
        .add_observer(on_move)
        .add_observer(on_jump)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.9, 0.4, 0.5), Vec2::new(40.0, 60.0)),
        actions!(Player[
            (
                Action::<Movement>::new(),
                DeadZone::default(),            // ignore tiny stick drift
                DeltaScale::default(),          // × delta time: frame-rate independent
                Scale::splat(300.0),            // speed
                Bindings::spawn((
                    Cardinal::wasd_keys(),      // W A S D → a Vec2
                    Axial::left_stick(),        // gamepad stick
                )),
            ),
            (
                Action::<Jump>::new(),
                bindings![KeyCode::Space, GamepadButton::South],
            ),
        ]),
    ));
}

fn on_move(movement: On<Fire<Movement>>, mut transforms: Query<&mut Transform>) {
    // `context` is the entity that owns the actions: the player.
    let mut transform = transforms.get_mut(movement.context).unwrap();
    transform.translation += movement.value.extend(0.0);
}

fn on_jump(jump: On<Start<Jump>>) {
    info!("jump pressed on {}", jump.context);
}
```

## Events

| Event | When |
|:--|:--|
| `Start<A>` | the action just started (e.g. key went down) |
| `Fire<A>` | every frame it is active |
| `Complete<A>` | it ended after firing |
| `Cancel<A>` | it ended before it fired (a hold released too early) |

Pick the event that matches the feel: `Start` for a jump, `Fire` for movement.

## Contexts: switch the controls

Add or remove the context component to change what the keys mean: walking → driving → menu. Several contexts can be layered with priorities (`context_layering` example).

## Alternatives

- [[leafwing-input-manager]]: the other mature action library; polling style.
- By hand: [[Input mapping]].

## Links

[crates.io](https://crates.io/crates/bevy_enhanced_input) · [docs.rs](https://docs.rs/bevy_enhanced_input) · [repo](https://github.com/simgine/bevy_enhanced_input) · examples: `basic_action_management`, `character_controller`, `keybinding_menu`.

## Next

[[Input]].
