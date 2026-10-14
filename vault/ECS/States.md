---
icon: waypoints
tags: [ecs]
---
# States

A **state** is "what the game is doing right now": `Menu`, `Playing`, `Paused`. Bevy stores it in a resource and runs systems when you *enter* or *leave* it. #ecs

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

## Try it

This is the real thing: Bevy's own `State`, `NextState` and `OnEnter` / `OnExit` schedules running in the preview. Press **Play** and watch the log: `OnExit(Menu)` runs first, then `OnEnter(Playing)`. **Back to menu** goes the other way.

```rust example=states
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    Playing,
}

#[derive(Component)]
struct MenuScreen;

#[derive(Component)]
struct Level;

fn spawn_menu(mut commands: Commands) {
    commands.spawn(MenuScreen);
    info!("OnEnter(Menu): spawn the menu");
}

fn despawn_menu(menus: Query<Entity, With<MenuScreen>>, mut commands: Commands) {
    for menu in &menus {
        commands.entity(menu).despawn();
    }
    info!("OnExit(Menu): despawn the menu");
}

fn spawn_level(mut commands: Commands) {
    commands.spawn(Level);
    info!("OnEnter(Playing): spawn the level");
}

fn despawn_level(levels: Query<Entity, With<Level>>, mut commands: Commands) {
    for level in &levels {
        commands.entity(level).despawn();
    }
    info!("OnExit(Playing): despawn the level");
}

fn plugin(app: &mut App) {
    app.init_state::<AppState>()
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(OnExit(AppState::Menu), despawn_menu)
        .add_systems(OnEnter(AppState::Playing), spawn_level)
        .add_systems(OnExit(AppState::Playing), despawn_level);
}

fn play(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Playing);
}
```

The menu entity exists only in the menu, the level only while playing. Each state brings what it needs and takes it away again; `DespawnOnExit` (below) does the removal for you.

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

With `bevy_dev_tools`, `bevy::dev_tools::states::log_transitions::<AppState>` prints every change. See [[Diagnostics and logging#Dev tools|Dev tools]].

## State and game design

- Use one `enum` per question: "which screen?" vs "paused?". Do not create `PlayingPaused`.
- Put `OnEnter` setup and `DespawnOnExit` cleanup *next to each other* in the same plugin ([[Project setup#Project structure|Project structure]]).
- Trigger the change from where the decision is made (a button's observer, a game-over system).

## Next

[[Change detection]].
