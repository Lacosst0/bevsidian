---
icon: map
tags: [2d]
---
# Tilemaps

A level made of a grid of repeating tiles: grass, water, walls. The map is just numbers; the pictures come from a tileset. #2d

## Grid maths first

A tile has a **cell** (column, row) and a **position** in the world. Convert between them yourself:

```rust example=tilemap
use bevy::prelude::*;

const TILE: f32 = 16.0;
const COLUMNS: u32 = 20;
const ROWS: u32 = 10;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Kind { Grass, Water, Sand, Rock }

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

/// The column and row of a tile.
#[derive(Component)]
struct Cell(UVec2);

/// Where the middle of a cell is in the world (the grid is centred on the screen).
fn cell_to_world(cell: UVec2) -> Vec2 {
    Vec2::new(
        (cell.x as f32 + 0.5) * TILE - COLUMNS as f32 * TILE / 2.0,
        (cell.y as f32 + 0.5) * TILE - ROWS as f32 * TILE / 2.0,
    )
}

fn spawn_grid(mut commands: Commands) {
    for y in 0..ROWS {
        for x in 0..COLUMNS {
            let cell = UVec2::new(x, y);
            let kind = if (x + y) % 7 == 0 { Kind::Water } else { Kind::Grass };
            commands.spawn((
                Cell(cell),
                kind,
                Sprite::from_color(kind.colour(), Vec2::splat(TILE - 1.0)),
                Transform::from_translation(cell_to_world(cell).extend(0.0)),
            ));
        }
    }
}
```

Every tile is an entity with a `Cell` and a `Kind`. The map *is* those components. Click to paint:

```rust example=tilemap
fn paint_tile(click: On<PointerClick>, mut tiles: Query<(&Cell, &mut Kind, &mut Sprite)>) {
    if let Ok((cell, mut kind, mut sprite)) = tiles.get_mut(click.entity) {
        debug!("painting the tile in column {}, row {}", cell.0.x, cell.0.y);
        *kind = kind.next();
        sprite.color = kind.colour();
    }
}
```

The reverse, "which cell is under this position?", is the same maths backwards:

```rust
use bevy::prelude::*;

const TILE: f32 = 16.0;

fn world_to_cell(position: Vec2) -> IVec2 {
    (position / TILE).floor().as_ivec2()
}
```

## The built-in `TilemapChunk`

One entity per tile is easy and flexible, but a big map (10 000 tiles) wants fewer entities. Bevy has `TilemapChunk`: one entity draws a whole grid from a *tileset image* (a stack of tile pictures) and a list of tile indices.

```rust
use bevy::{
    image::{ImageArrayLayout, ImageLoaderSettings},
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let chunk_size = UVec2::splat(64);                  // 64 x 64 tiles
    let tiles: Vec<Option<TileData>> = (0..chunk_size.element_product())
        .map(|i| Some(TileData::from_tileset_index((i % 4) as u16)))   // None = empty cell
        .collect();

    commands.spawn((
        TilemapChunk {
            chunk_size,
            tile_display_size: UVec2::splat(8),
            tileset: assets
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.array_layout = Some(ImageArrayLayout::RowCount { rows: 4 });
                })
                .load("textures/array_texture.png"),
            ..default()
        },
        TilemapChunkTileData(tiles),
    ));
}
```

Change a tile later by editing the list in `TilemapChunkTileData`.

## Bigger worlds

For huge or multi-layer maps, load levels drawn in an editor:

- [[bevy_ecs_ldtk]]: levels from LDtk, with entities and auto-tiling rules.
- [[bevy_ecs_tilemap]]: a full tilemap engine; also loads Tiled maps through add-ons.

## Collisions

Make "solid" a component and test the cell the player is moving into, see [[Physics]].

## Next

[[2D shapes and meshes]].
