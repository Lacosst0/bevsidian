//! `Resources.md`: a score that is global, and a system that raises it.

use bevy::prelude::*;

use crate::example::{Action, Example, Setup, View, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[update!(count_coins), update!(reset_when_high)],
    tick: None,
    actions: &[Action {
        name: "Score",
        label: "Collect a coin",
        run: collect,
    }],
    also: &[],
    view: View::Cards { fps: 3.0 },
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        registry.write().register::<Score>();
        registry.write().register::<Coin>();
        world.insert_resource(Score { points: 0 });
        world.spawn(Coin);
        world.spawn(Coin);
    })
}

/// A resource is a component of a singleton entity (Bevy 0.20), so a card shows it.
#[derive(Resource, Reflect)]
#[reflect(Resource, Component)]
struct Score {
    points: u32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Coin;

fn count_coins(mut score: ResMut<Score>, coins: Query<&Coin>) {
    score.points = coins.iter().count() as u32 * 10;
}

fn reset_when_high(mut score: ResMut<Score>) {
    if score.points > 50 {
        score.points = 0;
    }
}

fn collect(world: &mut World) {
    world.spawn(Coin);
}
