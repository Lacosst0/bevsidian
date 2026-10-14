---
lib: true
parent: Tilemap crates
icon: package
tags: [2d]
bevy: 0.19
crates: bevy_ecs_ldtk=0.15
---
# bevy_ecs_ldtk

Load levels drawn in **LDtk**, a friendly 2D level editor: tile layers, auto-tiling rules, and *entities with fields* that become Bevy components. Built on [[bevy_ecs_tilemap]]. #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.15` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_ecs_ldtk = "0.15"
```

## Load a project

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

## Entities from the editor

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

## Good with

[[Avian]] or [[bevy_rapier]] for colliders from the IntGrid layer; [[bevy_asset_loader]] for loading the project; [[bevy_light_2d]] for light.

## Links

[crates.io](https://crates.io/crates/bevy_ecs_ldtk) · [docs.rs](https://docs.rs/bevy_ecs_ldtk) · [book](https://trouv.github.io/bevy_ecs_ldtk/) · [repo](https://github.com/Trouv/bevy_ecs_ldtk)

## Next

[[Tilemaps]].
