//! `Commands.md`: spawning, despawning, inserting and removing with buttons.

use bevy::prelude::*;

use crate::example::{Action, Example, Setup, View, startup};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[startup!(spawn_slime)],
    tick: None,
    actions: &[
        Action {
            name: "spawn",
            label: "Spawn a slime",
            run: spawn,
        },
        Action {
            name: "despawn",
            label: "Despawn one",
            run: despawn,
        },
        Action {
            name: "insert",
            label: "Give a shield",
            run: insert,
        },
        Action {
            name: "remove",
            label: "Take it away",
            run: remove,
        },
    ],
    also: &[],
    view: View::Cards { fps: 2.0 },
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        let mut registry = registry.write();
        registry.register::<Name>();
        registry.register::<Slime>();
        registry.register::<Shield>();
    })
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Slime;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Shield(u32);

fn spawn_slime(mut commands: Commands) {
    commands.spawn((Name::new("Slime"), Slime));
}

fn spawn(world: &mut World) {
    world.commands().spawn((Name::new("Slime"), Slime));
}

fn slimes(world: &mut World) -> Vec<Entity> {
    world
        .query_filtered::<Entity, With<Slime>>()
        .iter(world)
        .collect()
}

fn despawn(world: &mut World) {
    if let Some(entity) = slimes(world).first() {
        world.commands().entity(*entity).despawn();
    }
}

fn insert(world: &mut World) {
    if let Some(entity) = slimes(world).first() {
        world.commands().entity(*entity).insert(Shield(5));
    }
}

fn remove(world: &mut World) {
    for entity in slimes(world) {
        world.commands().entity(entity).remove::<Shield>();
    }
}
