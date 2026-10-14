---
icon: gem
tags: [assets]
crates: serde={ version = "1", features = ["derive"] }; ron="0.12"
---
# Custom assets

Level files, item tables, dialogue: define your own asset type and Bevy loads, caches and hot-reloads it like an image. #assets

## An asset type

```rust
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

#[derive(Asset, TypePath, Debug, Deserialize)]
struct Level {
    name: String,
    enemies: Vec<(f32, f32)>,
}
```

## A loader

Tell Bevy how to turn bytes into your type, and which file extensions it handles:

```rust cont
use bevy::asset::{io::Reader, AssetLoader, LoadContext};

#[derive(Default, TypePath)]
struct LevelLoader;

impl AssetLoader for LevelLoader {
    type Asset = Level;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Level, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["level.ron"]
    }
}
```

## Register and use

```rust cont
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_asset::<Level>()
        .init_asset_loader::<LevelLoader>()
        .add_systems(Startup, start)
        .add_systems(Update, show)
        .run();
}

#[derive(Resource)]
struct CurrentLevel(Handle<Level>);

fn start(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CurrentLevel(asset_server.load("one.level.ron")));
}

fn show(current: Res<CurrentLevel>, levels: Res<Assets<Level>>) {
    if let Some(level) = levels.get(&current.0) {
        info!("level {}", level.name);
    }
}
```

`assets/one.level.ron`:

```text
(name: "First steps")
```

Edit the file while the game runs (with `file_watcher`) and `levels.get` returns the new data.

## Ready-made loaders

You often don't need to write one: [[Asset crates#bevy_common_assets|bevy_common_assets]] loads RON, JSON, TOML and more into any `serde` type.

## Next

[[Saving and loading]] for game saves.
