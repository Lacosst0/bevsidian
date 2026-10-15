//! `Randomness.md`: rocks scattered by a seeded generator. "Same seed again" rebuilds the
//! very same field; "New seed" a different one.
//!
//! The note uses `rand`'s `StdRng`; the preview has a tiny generator of its own, so it needs
//! no crates. The point is the same: one seed, one world.

use bevy::{ecs::system::RunSystemOnce, prelude::*};

use super::stage::{self, spawn_sky};
use crate::example::{Action, Example, Setup, startup};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[startup!(spawn_rocks)],
    tick: None,
    actions: &[
        Action {
            name: "spawn_rocks",
            label: "Same seed again",
            run: same_seed,
        },
        Action {
            name: "spawn_rocks",
            label: "New seed",
            run: new_seed,
        },
    ],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        world.insert_resource(Dice::seed_from_u64(2026));
    })
}

/// A seeded generator (xorshift): the same seed gives the same numbers, in the same order.
#[derive(Resource)]
struct Dice {
    seed: u64,
    state: u64,
}

impl Dice {
    fn seed_from_u64(seed: u64) -> Self {
        Self {
            seed,
            state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    fn random_range(&mut self, range: std::ops::Range<f32>) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        let unit = (self.state % 10_000) as f32 / 10_000.0;
        range.start + unit * (range.end - range.start)
    }
}

#[derive(Component)]
struct Rock;

fn spawn_rocks(mut commands: Commands, mut dice: ResMut<Dice>) {
    for _ in 0..12 {
        let x = dice.random_range(-140.0..140.0);
        let y = dice.random_range(-60.0..60.0);
        let size = dice.random_range(12.0..28.0);
        commands.spawn((
            Rock,
            Sprite::from_color(Color::srgb(0.5, 0.5, 0.55), Vec2::splat(size)),
            // Lower rocks in front, so the field has depth.
            Transform::from_xyz(x, y, 1.0 - y / 100.0),
        ));
    }
}

fn reroll(world: &mut World, seed: u64) {
    let rocks: Vec<Entity> = world
        .query_filtered::<Entity, With<Rock>>()
        .iter(world)
        .collect();
    for rock in rocks {
        world.despawn(rock);
    }
    world.insert_resource(Dice::seed_from_u64(seed));
    // The block's own system, run once more.
    let _ = world.run_system_once(spawn_rocks);
}

fn same_seed(world: &mut World) {
    let seed = world.resource::<Dice>().seed;
    reroll(world, seed);
}

fn new_seed(world: &mut World) {
    let seed = world.resource::<Dice>().seed + 1;
    reroll(world, seed);
}
