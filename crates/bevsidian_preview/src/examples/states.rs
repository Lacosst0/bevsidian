//! `States.md`: a menu and a level, with the real `State` machinery of `bevy_state`.
//! The buttons set `NextState`; the preview runs `StateTransition` every frame, as an `App`.

use bevy::{
    prelude::*,
    state::state::{
        FreelyMutableState, StateTransition, StateTransitionEvent, setup_state_transitions_in_world,
    },
};

use crate::example::{Action, Example, Log, Setup, View, scheduled};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[
        scheduled!(spawn_menu, OnEnter(AppState::Menu)),
        scheduled!(despawn_menu, OnExit(AppState::Menu)),
        scheduled!(spawn_level, OnEnter(AppState::Playing)),
        scheduled!(despawn_level, OnExit(AppState::Playing)),
    ],
    tick: Some(tick),
    actions: &[
        Action {
            name: "NextState",
            label: "Play",
            run: play,
        },
        Action {
            name: "AppState",
            label: "Back to menu",
            run: menu,
        },
    ],
    also: &[],
    view: View::Cards { fps: 2.0 },
};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
enum AppState {
    #[default]
    Menu,
    Playing,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct MenuScreen;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Level;

/// `app.init_state::<AppState>()` for a bare world.
fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>().clone();
        registry.write().register::<MenuScreen>();
        registry.write().register::<Level>();
        setup_state_transitions_in_world(world);
        world.insert_resource(State::new(AppState::Menu));
        world.init_resource::<NextState<AppState>>();
        world.init_resource::<Messages<StateTransitionEvent<AppState>>>();
        AppState::register_state(
            world
                .resource_mut::<Schedules>()
                .get_mut(StateTransition)
                .unwrap(),
        );
        world.write_message(StateTransitionEvent {
            exited: None,
            entered: Some(AppState::Menu),
            allow_same_state_transitions: true,
        });
    })
}

/// What `App` does between frames: swap message buffers, run the state transitions.
fn tick(world: &mut World) {
    world
        .resource_mut::<Messages<StateTransitionEvent<AppState>>>()
        .update();
    world.run_schedule(StateTransition);
}

fn spawn_menu(mut commands: Commands, mut log: ResMut<Log>) {
    commands.spawn(MenuScreen);
    log.say("OnEnter(Menu): spawn the menu");
}

fn despawn_menu(
    menus: Query<Entity, With<MenuScreen>>,
    mut commands: Commands,
    mut log: ResMut<Log>,
) {
    for menu in &menus {
        commands.entity(menu).despawn();
    }
    log.say("OnExit(Menu): despawn the menu");
}

fn spawn_level(mut commands: Commands, mut log: ResMut<Log>) {
    commands.spawn(Level);
    log.say("OnEnter(Playing): spawn the level");
}

fn despawn_level(levels: Query<Entity, With<Level>>, mut commands: Commands, mut log: ResMut<Log>) {
    for level in &levels {
        commands.entity(level).despawn();
    }
    log.say("OnExit(Playing): despawn the level");
}

fn play(world: &mut World) {
    go(world, AppState::Playing);
}

fn menu(world: &mut World) {
    go(world, AppState::Menu);
}

/// `next.set(state)`, then the transition a frame later; the preview does not play frames
/// by itself, so the button runs it at once.
fn go(world: &mut World, state: AppState) {
    world.resource_mut::<NextState<AppState>>().set(state);
    tick(world);
}
