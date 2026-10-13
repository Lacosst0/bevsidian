//! `Tilemaps.md`: a grid of tiles; click one to change what it is.

use bevy::prelude::*;

use super::stage::{self, SIZE, spawn_sky};
use crate::example::{Example, Setup, observer, startup};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[startup!(spawn_grid), observer!(paint_tile)],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|_| {})
}

const TILE: f32 = 16.0;
const COLUMNS: u32 = 20;
const ROWS: u32 = 10;

/// What a tile is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Grass,
    Water,
    Sand,
    Rock,
}

impl Kind {
    fn next(self) -> Self {
        match self {
            Kind::Grass => Kind::Water,
            Kind::Water => Kind::Sand,
            Kind::Sand => Kind::Rock,
            Kind::Rock => Kind::Grass,
        }
    }

    fn colour(self) -> Color {
        match self {
            Kind::Grass => Color::srgb(0.36, 0.7, 0.3),
            Kind::Water => Color::srgb(0.25, 0.5, 0.85),
            Kind::Sand => Color::srgb(0.9, 0.8, 0.5),
            Kind::Rock => Color::srgb(0.5, 0.5, 0.55),
        }
    }
}

/// The column and row of a tile; the origin of the grid is its bottom-left corner.
#[derive(Component)]
struct Cell(UVec2);

/// Where the middle of a cell is in the world.
fn cell_to_world(cell: UVec2) -> Vec2 {
    Vec2::new(
        (cell.x as f32 + 0.5) * TILE - SIZE.x / 2.0,
        (cell.y as f32 + 0.5) * TILE - SIZE.y / 2.0,
    )
}

fn spawn_grid(mut commands: Commands) {
    for y in 0..ROWS {
        for x in 0..COLUMNS {
            let cell = UVec2::new(x, y);
            let kind = if (x + y) % 7 == 0 {
                Kind::Water
            } else {
                Kind::Grass
            };
            commands.spawn((
                Cell(cell),
                kind,
                Sprite::from_color(kind.colour(), Vec2::splat(TILE - 1.0)),
                Transform::from_translation(cell_to_world(cell).extend(0.0)),
            ));
        }
    }
}

fn paint_tile(click: On<PointerClick>, mut tiles: Query<(&Cell, &mut Kind, &mut Sprite)>) {
    if let Ok((cell, mut kind, mut sprite)) = tiles.get_mut(click.entity) {
        debug!("painting the tile in column {}, row {}", cell.0.x, cell.0.y);
        *kind = kind.next();
        sprite.color = kind.colour();
    }
}
