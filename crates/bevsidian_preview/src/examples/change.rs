//! `Change detection.md`: only the health that changed gets reported.

use bevy::prelude::*;

use crate::example::{Action, Example, Log, Setup, View, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[
        startup!(spawn_team),
        update!(regenerate),
        update!(report_changes),
        update!(report_new),
    ],
    tick: None,
    actions: &[Action {
        name: "Hit",
        label: "Hit Bob",
        run: hit_bob,
    }],
    also: &[],
    view: View::Cards { fps: 2.0 },
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        registry.write().register::<Name>();
        registry.write().register::<Health>();
    })
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Health(u32);

fn spawn_team(mut commands: Commands) {
    commands.spawn((Name::new("Ann"), Health(10)));
    commands.spawn((Name::new("Bob"), Health(7)));
}

fn regenerate(mut query: Query<(&Name, &mut Health)>) {
    for (name, mut health) in &mut query {
        // Reading `health.0` does not count as a change; writing through `&mut` does.
        if health.0 < 10 && name.as_str() == "Ann" {
            health.0 += 1;
        }
    }
}

fn report_changes(query: Query<(&Name, &Health), Changed<Health>>, mut log: ResMut<Log>) {
    for (name, health) in &query {
        log.say(format!("{name}'s health changed to {}", health.0));
    }
}

fn report_new(query: Query<&Name, Added<Health>>, mut log: ResMut<Log>) {
    for name in &query {
        log.say(format!("{name} got a Health component"));
    }
}

fn hit_bob(world: &mut World) {
    let mut query = world.query::<(&Name, &mut Health)>();
    for (name, mut health) in query.iter_mut(world) {
        if name.as_str() == "Bob" {
            health.0 = health.0.saturating_sub(3);
        }
    }
}
