---
parent: Input
icon: sliders-horizontal
tags: [input]
crates: bevy_enhanced_input={ git = "https://github.com/simgine/bevy_enhanced_input" }; leafwing-input-manager={ git = "https://github.com/Leafwing-Studios/leafwing-input-manager" }
---
# Input crates

Two crates for actions and rebinding. Both build on #0.20 from their git `main` branch.

| | [[bevy_enhanced_input]] | [[leafwing-input-manager]] |
|:--|:--|:--|
| Bevy | #0.20 (git) | #0.20 (git) |
| Model | contexts with actions, read by observers | an action enum, an `InputMap`, an `ActionState` |
| Switching schemes | layered contexts (on foot, driving, menu) | swap or merge maps yourself |
| Feels like | Unreal's Enhanced Input | the long-standing Bevy default |

> [!TIP]
> - Control schemes that switch often: [[bevy_enhanced_input]].
> - One action enum, rebinding and gamepads: [[leafwing-input-manager]].
> - A handful of keys: skip both and read input directly ([[Input]]).

## The same jump, both ways

Space or the gamepad's South button makes the player jump:

```rust variant=bevy_enhanced_input fresh
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_systems(Startup, spawn_player)
        .add_observer(jump)
        .run();
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        actions!(Player[(
            Action::<Jump>::new(),
            bindings![KeyCode::Space, GamepadButton::South],
        )]),
    ));
}

// Runs when the action starts: no polling.
fn jump(jump: On<Start<Jump>>) {
    info!("{} jumps", jump.context);
}
```

```rust variant=leafwing-input-manager fresh
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Jump,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InputManagerPlugin::<Action>::default()))
        .add_systems(Startup, spawn_player)
        .add_systems(Update, jump)
        .run();
}

fn spawn_player(mut commands: Commands) {
    let mut map = InputMap::default();
    map.insert(Action::Jump, KeyCode::Space);
    map.insert(Action::Jump, GamepadButton::South);
    commands.spawn((Player, map));
}

// Asked every frame.
fn jump(player: Single<&ActionState<Action>, With<Player>>) {
    if player.just_pressed(&Action::Jump) {
        info!("jump!");
    }
}
```

## Next

[[Input mapping]], [[Input]].
