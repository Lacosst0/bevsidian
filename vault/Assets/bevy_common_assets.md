---
icon: package
tags: [assets]
crates: bevy_common_assets={ version = "0.18", features = ["ron"] }; serde={ version = "1", features = ["derive"] }
---
# bevy_common_assets

Load **RON, JSON, TOML, YAML, CSV, XML, MessagePack and more** straight into your own `serde` types as Bevy assets. No loader code: one plugin per file format. #assets

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_common_assets 0.18` supports **Bevy 0.20**.

```toml
bevy_common_assets = { version = "0.18", features = ["ron"] }   # pick your formats
serde = { version = "1", features = ["derive"] }
```

Write a [[Custom assets|custom asset loader]] once by hand to understand it; then use this.

## RON in five lines

```rust
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

#[derive(serde::Deserialize, Asset, TypePath)]
struct Level {
    name: String,
    enemies: Vec<[f32; 2]>,
}

#[derive(Resource)]
struct CurrentLevel(Handle<Level>);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RonAssetPlugin::<Level>::new(&["level.ron"])))
        .add_systems(Startup, load)
        .add_systems(Update, spawn_when_loaded)
        .run();
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CurrentLevel(asset_server.load("one.level.ron")));
}

fn spawn_when_loaded(
    mut commands: Commands,
    current: Res<CurrentLevel>,
    mut levels: ResMut<Assets<Level>>,
) {
    if let Some(level) = levels.remove(current.0.id()) {
        info!("loaded {}", level.name);
        for [x, y] in level.enemies {
            commands.spawn((Name::new("Enemy"), Transform::from_xyz(x, y, 0.0)));
        }
    }
}
```

`assets/one.level.ron`:

```text
(name: "Cave", enemies: [(10.0, 20.0), (-30.0, 5.0)])
```

## Why this is the best tuning tool

Combine with `file_watcher` ([[Hot reloading]]): edit the `.ron` file, save, and the numbers in the running game change. Put enemy stats, item tables, dialogue and level layouts in such files, not in Rust.

## Formats

| Feature | Plugin |
|:--|:--|
| `ron` | `RonAssetPlugin` |
| `json` | `JsonAssetPlugin` |
| `toml` | `TomlAssetPlugin` |
| `yaml` | `YamlAssetPlugin` |
| `csv` | `CsvAssetPlugin` |
| `xml`, `msgpack`, `postcard`, `cbor` | matching plugins |

## Links

[crates.io](https://crates.io/crates/bevy_common_assets) · [docs.rs](https://docs.rs/bevy_common_assets) · [repo](https://github.com/NiklasEi/bevy_common_assets)

## Next

[[Custom assets]], [[bevy_asset_loader]].
