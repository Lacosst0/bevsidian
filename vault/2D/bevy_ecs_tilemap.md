---
lib: true
parent: Tilemap crates
icon: package
tags: [2d]
bevy: 0.19
crates: bevy_ecs_tilemap=0.19
---
# bevy_ecs_tilemap

A **tilemap engine**: every tile is an entity, drawn in efficient chunks from a tileset image. Square, isometric and hexagonal grids, many layers, animated tiles. #2d

#0.19 · for a basic tilemap, Bevy has [[Tilemaps|`TilemapChunk`]]

```toml
bevy_ecs_tilemap = "0.19"
```

## A filled map

```rust
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let texture: Handle<Image> = asset_server.load("tiles.png");
    let map_size = TilemapSize { x: 32, y: 32 };

    // Make the map entity first: every tile needs to know which map it belongs to.
    let tilemap_entity = commands.spawn_empty().id();
    let mut storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let position = TilePos { x, y };
            let tile = commands
                .spawn(TileBundle {
                    position,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex((x + y) % 4),   // which picture
                    ..default()
                })
                .id();
            storage.set(&position, tile);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: tile_size.into(),
        map_type: TilemapType::default(),     // Square, Isometric(..), Hexagon(..)
        size: map_size,
        storage,
        texture: TilemapTexture::Single(texture),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..default()
    });
}
```

A tile is an entity with a `TilePos` and `TileTextureIndex`; change a tile at runtime by changing its components ([[Change detection]]).

## When to choose it

- Large maps, isometric or hex grids, several layers.
- You want tile *entities* so tiles can have behaviour (damage, ownership).
- For the level *editor*, add [[bevy_ecs_ldtk]] on top.
- For a small map, hand-made sprites or Bevy's own chunk ([[Tilemaps]]) are simpler.

## Links

[crates.io](https://crates.io/crates/bevy_ecs_tilemap) · [docs.rs](https://docs.rs/bevy_ecs_tilemap) · [repo](https://github.com/StarArawn/bevy_ecs_tilemap)

## Next

[[Tilemaps]].
