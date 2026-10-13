//! `Relationships and hierarchy.md`: a ship with a crew, and what happens when the ship goes.

use bevy::prelude::*;

use crate::example::{Action, Example, Setup, View, startup};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[startup!(spawn_ship)],
    tick: None,
    actions: &[
        Action {
            name: "ChildOf",
            label: "Add a sailor",
            run: add_sailor,
        },
        Action {
            name: "despawn",
            label: "Sink the ship",
            run: sink,
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
        registry.register::<ChildOf>();
        registry.register::<Children>();
        registry.register::<Ship>();
    })
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Ship;

fn spawn_ship(mut commands: Commands) {
    commands.spawn((
        Name::new("Ship"),
        Ship,
        children![Name::new("Captain"), Name::new("Cook")],
    ));
}

fn add_sailor(world: &mut World) {
    let ship = world
        .query_filtered::<Entity, With<Ship>>()
        .iter(world)
        .next();
    if let Some(ship) = ship {
        world.commands().spawn((Name::new("Sailor"), ChildOf(ship)));
    }
}

fn sink(world: &mut World) {
    let ship = world
        .query_filtered::<Entity, With<Ship>>()
        .iter(world)
        .next();
    if let Some(ship) = ship {
        world.commands().entity(ship).despawn();
    }
}
