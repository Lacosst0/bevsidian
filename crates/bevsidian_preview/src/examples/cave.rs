//! `Procedural generation.md`: a cave grown by cellular automata. Random noise, then a few
//! smoothing steps; "New cave" starts again with a new seed.
//!
//! The preview uses a tiny generator of its own (the note uses `rand`); the algorithm is the same.

// Map sizes are tiny constants: the casts cannot wrap.
#![allow(clippy::cast_possible_wrap)]

use bevy::prelude::*;

use super::stage::{self, spawn_sky};
use crate::example::{Action, Example, Setup, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky), startup!(spawn_tiles), update!(draw)],
    functions: &[startup!(random_fill), update!(smooth)],
    tick: None,
    actions: &[Action {
        name: "smooth",
        label: "New cave",
        run: new_cave,
    }],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        // Noise from the first frame (`draw` runs before the block's `random_fill`).
        let mut cave = Cave::empty();
        cave.fill();
        world.insert_resource(cave);
    })
}

const WIDTH: usize = 40;
const HEIGHT: usize = 20;
const TILE: f32 = 8.0;
const STEPS: u32 = 5;
const WALL: Color = Color::srgb(0.42, 0.36, 0.5);
const FLOOR: Color = Color::srgb(0.17, 0.16, 0.23);

#[derive(Resource)]
struct Cave {
    walls: Vec<bool>,
    seed: u64,
    steps: u32,
    wait: f32,
}

impl Cave {
    fn empty() -> Self {
        Self {
            walls: vec![false; WIDTH * HEIGHT],
            seed: 7,
            steps: 0,
            wait: 0.0,
        }
    }

    fn fill(&mut self) {
        let mut state = self.seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        for wall in &mut self.walls {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *wall = (state % 100) < 46;
        }
        self.steps = 0;
    }

    fn wall_count(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for dy in -1..=1_i32 {
            for dx in -1..=1_i32 {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                let outside = nx < 0 || ny < 0 || nx >= WIDTH as i32 || ny >= HEIGHT as i32;
                if (dx, dy) != (0, 0) && (outside || self.walls[ny as usize * WIDTH + nx as usize])
                {
                    count += 1;
                }
            }
        }
        count
    }

    fn smooth(&mut self) {
        let next: Vec<bool> = (0..WIDTH * HEIGHT)
            .map(|i| self.wall_count(i % WIDTH, i / WIDTH) >= 5)
            .collect();
        self.walls = next;
        self.steps += 1;
    }
}

#[derive(Component)]
struct Tile(usize);

fn new_cave(world: &mut World) {
    let mut cave = world.resource_mut::<Cave>();
    cave.seed += 1;
    cave.fill();
}

fn spawn_tiles(mut commands: Commands) {
    for i in 0..WIDTH * HEIGHT {
        let (x, y) = ((i % WIDTH) as f32, (i / WIDTH) as f32);
        commands.spawn((
            Tile(i),
            Sprite::from_color(FLOOR, Vec2::splat(TILE)),
            Transform::from_xyz(
                (x + 0.5) * TILE - stage::SIZE.x / 2.0,
                (y + 0.5) * TILE - stage::SIZE.y / 2.0,
                0.0,
            ),
        ));
    }
}

fn random_fill(mut cave: ResMut<Cave>) {
    cave.fill();
}

fn smooth(time: Res<Time>, mut cave: ResMut<Cave>) {
    cave.wait += time.delta_secs();
    if cave.wait >= 0.5 && cave.steps < STEPS {
        cave.wait = 0.0;
        cave.smooth();
    }
}

fn draw(cave: Res<Cave>, mut tiles: Query<(&Tile, &mut Sprite)>) {
    for (tile, mut sprite) in &mut tiles {
        sprite.color = if cave.walls[tile.0] { WALL } else { FLOOR };
    }
}
