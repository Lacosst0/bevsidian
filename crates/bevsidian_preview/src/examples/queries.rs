//! `Queries.md`: a small party, and the queries that pick members out of it.

use bevy::prelude::*;

use crate::example::{Action, Example, Function, Log, Setup, View, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: FUNCTIONS,
    tick: None,
    actions: &[Action {
        name: "Poison",
        label: "Poison the goblin",
        run: poison_goblin,
    }],
    also: &[],
    view: View::Cards { fps: 2.0 },
};

const FUNCTIONS: &[Function] = &[
    startup!(spawn_party),
    update!(poison_damage),
    update!(rest),
    update!(report),
];

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        let mut registry = registry.write();
        registry.register::<Name>();
        registry.register::<Hero>();
        registry.register::<Enemy>();
        registry.register::<Health>();
        registry.register::<Poison>();
    })
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Hero;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Enemy;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Health(u32);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Poison(u32);

fn spawn_party(mut commands: Commands) {
    commands.spawn((Name::new("Hero"), Hero, Health(30)));
    commands.spawn((Name::new("Goblin"), Enemy, Health(12)));
    commands.spawn((Name::new("Wolf"), Enemy, Health(18), Poison(2)));
}

fn poison_damage(mut query: Query<(&mut Health, &Poison)>) {
    for (mut health, poison) in &mut query {
        health.0 = health.0.saturating_sub(poison.0);
    }
}

fn rest(mut query: Query<&mut Health, Without<Poison>>) {
    for mut health in &mut query {
        health.0 = (health.0 + 1).min(30);
    }
}

fn report(query: Query<(&Name, Option<&Poison>)>, mut log: ResMut<Log>) {
    for (name, poison) in &query {
        match poison {
            Some(poison) => log.say(format!("{name} is poisoned ({})", poison.0)),
            None => log.say(format!("{name} is fine")),
        }
    }
}

/// The button: poisons the first enemy that is not poisoned yet.
fn poison_goblin(world: &mut World) {
    let target = world
        .query_filtered::<Entity, (With<Enemy>, Without<Poison>)>()
        .iter(world)
        .next();
    if let Some(entity) = target {
        world.entity_mut(entity).insert(Poison(3));
    }
}
