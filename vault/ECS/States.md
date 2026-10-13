---
icon: waypoints
tags: [ecs, intermediate]
---
# States

A **state** is "what the game is doing right now": `Menu`, `Playing`, `Paused`. Bevy stores it in a resource and runs systems when you *enter* or *leave* it. #ecs #intermediate

## Define and use

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    Playing,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()                       // starts at the #[default]
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(OnExit(AppState::Menu), despawn_menu)
        .add_systems(OnEnter(AppState::Playing), spawn_level)
        .add_systems(Update, play.run_if(in_state(AppState::Playing)))
        .run();
}

fn spawn_menu() {}
fn despawn_menu() {}
fn spawn_level() {}
fn play() {}
```

| Piece | Meaning |
|:--|:--|
| `OnEnter(S)` | runs once when entering `S` |
| `OnExit(S)` | runs once when leaving `S`, **before** the next `OnEnter` |
| `in_state(S)` | [[Run conditions|run condition]]: true while in `S` |
| `State<AppState>` | resource holding the current value |

## Changing state

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState { #[default] Menu, Playing }

fn start(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Playing);
}
```

The change happens the next time Bevy runs the `StateTransition` schedule (once a frame), not instantly. Setting the state you are already in does nothing visible by default.

## Clean up for free

Mark entities with `DespawnOnExit` and they vanish when the state ends:

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState { #[default] Menu, Playing }

fn spawn_level(mut commands: Commands) {
    commands.spawn((Name::new("Level"), DespawnOnExit(AppState::Playing)));
}
```

`DespawnOnEnter` is its opposite. Compared to hand-written cleanup systems, this cannot be forgotten.

## Sub-states

A state that only exists inside another one. Perfect for pause:

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState { #[default] Menu, Playing }

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::Playing)]
enum Pause {
    #[default]
    Running,
    Paused,
}

fn main() {
    App::new()
        .init_state::<AppState>()
        .add_sub_state::<Pause>()
        .add_systems(Update, physics.run_if(in_state(Pause::Running)))
        .run();
}

fn physics() {}
```

Outside `Playing`, the `Pause` resource does not exist at all, so a system cannot get it wrong.

## Computed states

When a state is *derived* from others (e.g. "is the game paused or in a menu?") compute it:

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState { #[default] Menu, Playing }

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct InGame;

impl ComputedStates for InGame {
    type SourceStates = AppState;

    fn compute(source: AppState) -> Option<Self> {
        matches!(source, AppState::Playing).then_some(InGame)
    }
}

fn main() {
    App::new().init_state::<AppState>().add_computed_state::<InGame>();
}
```

## Debug transitions

With `bevy_dev_tools`, `bevy::dev_tools::states::log_transitions::<AppState>` prints every change. See [[Dev tools]].

## State and game design

- Use one `enum` per question: "which screen?" vs "paused?". Do not create `PlayingPaused`.
- Put `OnEnter` setup and `DespawnOnExit` cleanup *next to each other* in the same plugin ([[Project structure]]).
- Trigger the change from where the decision is made (a button's observer, a game-over system).

## Next

[[Change detection]].
