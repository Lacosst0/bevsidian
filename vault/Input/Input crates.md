---
icon: package
tags: [input]
crates: bevy_enhanced_input={ git = "https://github.com/simgine/bevy_enhanced_input" }; leafwing-input-manager={ git = "https://github.com/Leafwing-Studios/leafwing-input-manager" }
---
# Input crates

Two crates for actions and rebinding, both on the way to Bevy 0.20. Start with [[Input mapping]] to choose.

## bevy_enhanced_input

An **observer-based input manager**: you describe *contexts* (on foot, in a car) with *actions* (Move, Jump) and *bindings* (keys, sticks); your game just observes the actions. Inspired by Unreal's Enhanced Input. #input

> [!IMPORTANT]
> **Bevy 0.20 status** (checked 2026-10-13): the crates.io release `0.26` targets Bevy 0.19. The `main` branch already targets **0.20**: depend on it with `git` until the next release.

```toml
[dependencies]
bevy = "0.20"
bevy_enhanced_input = { git = "https://github.com/simgine/bevy_enhanced_input" }
```

### Why choose it

- Input is **data on entities**, so every player or vehicle has its own context ([[Relationships and hierarchy|entities with children]]).
- **Modifiers** shape raw input: dead zones, smoothing, scaling, negating, delta-time scaling.
- **Conditions** decide when an action fires: pressed, held for 0.5 s, tapped, chorded with another key.
- Actions arrive as [[Events and observers|observer events]] (`Start`, `Fire`, `Complete`...), no polling.
- Rebinding screens are easy: bindings are components you can change.

### The four ideas

| Idea | In code |
|:--|:--|
| **Action**: what the player wants | a type with `#[derive(InputAction)]` and an output type (`bool`, `f32`, `Vec2`) |
| **Context**: a set of actions that are active together | a component registered with `add_input_context::<T>()` |
| **Binding**: a key/stick/button for an action | `bindings![KeyCode::Space]`, `Cardinal::wasd_keys()`, `Axial::left_stick()` |
| **Modifier / condition**: shape or gate the input | components on the action: `Scale`, `DeadZone`, `DeltaScale`... |

### A player that moves and jumps

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

### Events

| Event | When |
|:--|:--|
| `Start<A>` | the action just started (e.g. key went down) |
| `Fire<A>` | every frame it is active |
| `Complete<A>` | it ended after firing |
| `Cancel<A>` | it ended before it fired (a hold released too early) |

Pick the event that matches the feel: `Start` for a jump, `Fire` for movement.

### Contexts: switch the controls

Add or remove the context component to change what the keys mean: walking → driving → menu. Several contexts can be layered with priorities (`context_layering` example).

### Alternatives

- [[Input crates#leafwing-input-manager|leafwing-input-manager]]: the other mature action library; polling style.
- By hand: [[Input mapping]].

### Links

[crates.io](https://crates.io/crates/bevy_enhanced_input) · [docs.rs](https://docs.rs/bevy_enhanced_input) · [repo](https://github.com/simgine/bevy_enhanced_input) · examples: `basic_action_management`, `character_controller`, `keybinding_menu`.

## leafwing-input-manager

Map physical inputs to your own **action enum**, then ask `ActionState` whether an action is pressed. A long-established crate with a very full feature set. #input

> [!IMPORTANT]
> **Bevy 0.20 status** (checked 2026-10-13): release `0.21` is for Bevy 0.19; the `main` branch is on **0.20**. Use `git` until it is released.

```toml
leafwing-input-manager = { git = "https://github.com/Leafwing-Studios/leafwing-input-manager" }
```

### The idea

1. An `enum` of everything the player can *do* (derive `Actionlike`).
2. An `InputMap<Action>` component: which keys/buttons/sticks trigger each action.
3. An `ActionState<Action>` component (added for you): the current state of each action. Systems read it.

```rust
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InputManagerPlugin::<PlayerAction>::default()))
        .add_systems(Startup, spawn_player)
        .add_systems(Update, act)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    #[actionlike(DualAxis)]
    Run,
    Jump,
    UseItem,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    let mut map = InputMap::default();
    // Gamepad
    map.insert_dual_axis(PlayerAction::Run, GamepadStick::LEFT);
    map.insert(PlayerAction::Jump, GamepadButton::South);
    // Keyboard and mouse
    map.insert_dual_axis(PlayerAction::Run, VirtualDPad::wasd());
    map.insert(PlayerAction::Jump, KeyCode::Space);
    map.insert(PlayerAction::UseItem, MouseButton::Left);

    commands.spawn((Player, map));
}

fn act(player: Single<&ActionState<PlayerAction>, With<Player>>) {
    let run = player.clamped_axis_pair(&PlayerAction::Run);
    if run != Vec2::ZERO {
        info!("moving {run}");
    }
    if player.just_pressed(&PlayerAction::Jump) {
        info!("jump!");
    }
}
```

### What it adds over reading keys yourself

- **Remapping** at runtime (change the `InputMap`, save it).
- **Chords** (`Ctrl+S`), **press duration** (`action_state.current_duration`), double taps.
- **Axes** with dead zones and sensitivity processing.
- **Several players**: one entity per player, each with its own map and gamepad.
- **Clash handling**: `Ctrl+S` doesn't also fire `S`.
- Sending actions over the network (`send_actions_over_network` example).

### Choosing

| | [[Input crates#bevy_enhanced_input|bevy_enhanced_input]] | leafwing-input-manager |
|:--|:--|:--|
| Style | observer events with `On<Fire<A>>` | poll `ActionState` |
| Actions are | types with their own output | enum variants |
| Rebinding | components | `InputMap` methods |
| Maturity | newer, Unreal-inspired | older, widely used |

Either beats hand-written key checks as soon as you want a settings screen.

### Links

[crates.io](https://crates.io/crates/leafwing-input-manager) · [docs.rs](https://docs.rs/leafwing-input-manager) · [repo](https://github.com/Leafwing-Studios/leafwing-input-manager)
