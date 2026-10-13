//! `Events and observers.md`: a door that gets knocked on. Every observer says what it saw.

use bevy::prelude::*;

use crate::example::{Action, Example, Log, Setup, View, observer, startup};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[
        startup!(spawn_house),
        observer!(on_knock),
        observer!(on_ring),
        observer!(on_open),
        observer!(on_door_added),
    ],
    tick: None,
    actions: &[
        Action {
            name: "Knock",
            label: "Knock on the front door",
            run: knock,
        },
        Action {
            name: "Ring",
            label: "Ring the bell",
            run: ring,
        },
        Action {
            name: "Open",
            label: "Open the back door",
            run: open,
        },
    ],
    also: &[],
    view: View::Cards { fps: 2.0 },
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        registry.write().register::<Name>();
        registry.write().register::<Door>();
    })
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Door;

/// Aimed at one door.
#[derive(EntityEvent)]
struct Knock {
    entity: Entity,
}

/// Aimed at nobody in particular.
#[derive(Event)]
struct Ring {
    times: u32,
}

#[derive(EntityEvent)]
struct Open {
    entity: Entity,
}

fn spawn_house(mut commands: Commands) {
    commands.spawn((Name::new("Front door"), Door));
    commands.spawn((Name::new("Back door"), Door));
}

fn on_knock(knock: On<Knock>, names: Query<&Name>, mut log: ResMut<Log>) {
    if let Ok(name) = names.get(knock.entity) {
        log.say(format!("knock, knock: {name} heard it"));
    }
}

fn on_ring(ring: On<Ring>, mut log: ResMut<Log>) {
    log.say(format!("the bell rings {} times", ring.times));
}

fn on_open(open: On<Open>, mut commands: Commands, mut log: ResMut<Log>) {
    log.say("the back door opens");
    commands.entity(open.entity).remove::<Door>();
}

fn on_door_added(add: On<Add<Door>>, names: Query<&Name>, mut log: ResMut<Log>) {
    let name = names.get(add.entity).map_or("a door", Name::as_str);
    log.say(format!("{name} was built"));
}

fn door(world: &mut World, name: &str) -> Option<Entity> {
    world
        .query_filtered::<(Entity, &Name), With<Door>>()
        .iter(world)
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
}

fn knock(world: &mut World) {
    if let Some(entity) = door(world, "Front door") {
        world.commands().trigger(Knock { entity });
    }
}

fn ring(world: &mut World) {
    world.commands().trigger(Ring { times: 2 });
}

fn open(world: &mut World) {
    if let Some(entity) = door(world, "Back door") {
        world.commands().trigger(Open { entity });
    }
}
