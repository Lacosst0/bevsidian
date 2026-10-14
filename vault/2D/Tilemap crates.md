---
lib: true
icon: package
tags: [2d]
bevy: 0.19
crates: bevy_ecs_tilemap=0.19; bevy_ecs_ldtk=0.15
---
# Tilemap crates

Two crates for big tile worlds: a fast tilemap renderer, and an importer for levels drawn in the LDtk editor. Both are pending for Bevy 0.20. For small maps see [[Tilemaps]].

## bevy_ecs_tilemap

A **tilemap engine**: every tile is an entity, drawn in efficient chunks from a tileset image. Square, isometric and hexagonal grids, many layers, animated tiles. #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.19` targets **Bevy 0.19**. Bevy 0.20 itself now ships a basic [[Tilemaps|`TilemapChunk`]]. Compiled against 0.19.

```toml
bevy_ecs_tilemap = "0.19"
```

### A filled map

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

### When to choose it

- Large maps, isometric or hex grids, several layers.
- You want tile *entities* so tiles can have behaviour (damage, ownership).
- For the level *editor*, add [[Tilemap crates#bevy_ecs_ldtk|bevy_ecs_ldtk]] on top.
- For a small map, hand-made sprites or Bevy's own chunk ([[Tilemaps]]) are simpler.

### Links

[crates.io](https://crates.io/crates/bevy_ecs_tilemap) · [docs.rs](https://docs.rs/bevy_ecs_tilemap) · [repo](https://github.com/StarArawn/bevy_ecs_tilemap)

## bevy_ecs_ldtk

Load levels drawn in **LDtk**, a friendly 2D level editor: tile layers, auto-tiling rules, and *entities with fields* that become Bevy components. Built on [[Tilemap crates#bevy_ecs_tilemap|bevy_ecs_tilemap]]. #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.15` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_ecs_ldtk = "0.15"
```

### Load a project

Design the level in [LDtk](https://ldtk.io), save `assets/world.ldtk`, then:

```rust
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::index(0))      // which level to show
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("world.ldtk").into(),
        ..default()
    });
}
```

### Entities from the editor

Place a "Player" entity in LDtk; register a bundle for it and it spawns with your components:

```rust
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Component, Default)]
struct Player;

#[derive(Bundle, LdtkEntity, Default)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet]
    sprite: Sprite,
}

fn register(app: &mut App) {
    app.register_ldtk_entity::<PlayerBundle>("Player");   // the entity name in LDtk
}
```

Designers move the player spawn point without touching code; fields set in the editor (health, patrol path) map to component fields.

### Good with

[[Avian]] or [[bevy_rapier]] for colliders from the IntGrid layer; [[Asset crates#bevy_asset_loader|bevy_asset_loader]] for loading the project; [[2D lighting crates#bevy_light_2d|bevy_light_2d]] for light.

### Links

[crates.io](https://crates.io/crates/bevy_ecs_ldtk) · [docs.rs](https://docs.rs/bevy_ecs_ldtk) · [book](https://trouv.github.io/bevy_ecs_ldtk/) · [repo](https://github.com/Trouv/bevy_ecs_ldtk)
