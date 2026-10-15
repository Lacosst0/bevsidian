---
lib: true
parent: Input crates
icon: package
tags: [input]
crates: leafwing-input-manager={ git = "https://github.com/Leafwing-Studios/leafwing-input-manager" }
---
# leafwing-input-manager

Map physical inputs to your own **action enum**, then ask `ActionState` whether an action is pressed. A long-established crate with a very full feature set. #input

#0.20 · from git

```toml
leafwing-input-manager = { git = "https://github.com/Leafwing-Studios/leafwing-input-manager" }
```

## The idea

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

## What it adds over reading keys yourself

- **Remapping** at runtime (change the `InputMap`, save it).
- **Chords** (`Ctrl+S`), **press duration** (`action_state.current_duration`), double taps.
- **Axes** with dead zones and sensitivity processing.
- **Several players**: one entity per player, each with its own map and gamepad.
- **Clash handling**: `Ctrl+S` doesn't also fire `S`.
- Sending actions over the network (`send_actions_over_network` example).

## Choosing

| | [[bevy_enhanced_input]] | leafwing-input-manager |
|:--|:--|:--|
| Style | observer events with `On<Fire<A>>` | poll `ActionState` |
| Actions are | types with their own output | enum variants |
| Rebinding | components | `InputMap` methods |
| Maturity | newer, Unreal-inspired | older, widely used |

Either beats hand-written key checks as soon as you want a settings screen.

## Links

[crates.io](https://crates.io/crates/leafwing-input-manager) · [docs.rs](https://docs.rs/leafwing-input-manager) · [repo](https://github.com/Leafwing-Studios/leafwing-input-manager)

## Next

[[Input mapping]].
