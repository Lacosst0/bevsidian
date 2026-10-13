//! `Run conditions.md`: enemies move only while the game is not paused.

use bevy::prelude::*;

use crate::example::{Action, Example, Log, Setup, View, startup, update, update_if};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[
        startup!(spawn_enemies),
        update_if!(move_enemies, not_paused),
        update_if!(announce, resource_changed::<Paused>),
        update!(always_runs),
    ],
    tick: None,
    actions: &[Action {
        name: "Paused",
        label: "Toggle pause",
        run: toggle,
    }],
    also: &[],
    view: View::Cards { fps: 2.0 },
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        registry.write().register::<Paused>();
        registry.write().register::<Enemy>();
        registry.write().register::<Distance>();
        world.insert_resource(Paused(false));
    })
}

#[derive(Resource, Reflect)]
#[reflect(Resource, Component)]
struct Paused(bool);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Enemy;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Distance(u32);

fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

fn spawn_enemies(mut commands: Commands) {
    commands.spawn((Enemy, Distance(0)));
}

fn move_enemies(mut enemies: Query<&mut Distance, With<Enemy>>) {
    for mut distance in &mut enemies {
        distance.0 += 1;
    }
}

fn announce(paused: Res<Paused>, mut log: ResMut<Log>) {
    log.say(format!("Paused changed to {}", paused.0));
}

fn always_runs(mut log: ResMut<Log>, paused: Res<Paused>) {
    if paused.0 {
        log.say("always_runs: still running while paused");
    }
}

fn toggle(world: &mut World) {
    let mut paused = world.resource_mut::<Paused>();
    paused.0 = !paused.0;
}
