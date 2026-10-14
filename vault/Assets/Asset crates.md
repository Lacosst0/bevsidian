---
lib: true
icon: package
tags: [assets]
crates: bevy_asset_loader={ version = "0.28", features = ["2d"] }; bevy_common_assets={ version = "0.18", features = ["ron"] }; serde={ version = "1", features = ["derive"] }
---
# Asset crates

Two crates that make assets less manual: declare what to load in a struct, and load data files (RON, JSON, TOML) as assets. See [[Assets]] and [[Custom assets]].

## bevy_asset_loader

Describe all your assets **in a struct**; the crate loads them during a loading [[States|state]], then moves on and gives you the finished resource. Replaces hand-written preloading and `is_loaded` checks ([[Assets]]). #assets

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_asset_loader 0.28` supports **Bevy 0.20**.

```toml
bevy_asset_loader = { version = "0.28", features = ["2d"] }   # "2d" adds texture atlas support, "3d" materials
```

### An asset collection

```rust
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Screen {
    #[default]
    Loading,
    Playing,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,

    #[asset(path = "audio/theme.ogg")]
    theme: Handle<AudioSource>,

    // Cut a sprite sheet into frames while loading.
    #[asset(texture_atlas_layout(tile_size_x = 24, tile_size_y = 24, columns = 7, rows = 1))]
    layout: Handle<TextureAtlasLayout>,

    // Load a whole folder.
    #[asset(path = "images/tiles", collection(typed))]
    tiles: Vec<Handle<Image>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<Screen>()
        .add_loading_state(
            LoadingState::new(Screen::Loading)
                .continue_to_state(Screen::Playing)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(Screen::Playing), spawn_player)
        .run();
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(assets.player.clone()));
}
```

When the state becomes `Playing`, every handle in `GameAssets` is loaded, and `Res<GameAssets>` exists. No "is it ready yet?" code anywhere else.

### Useful attributes

| Attribute | Does |
|:--|:--|
| `#[asset(path = "...")]` | one file |
| `#[asset(paths("a.png", "b.png"), collection)]` | a list |
| `collection(typed, mapped)` | a folder as `HashMap<String, Handle<T>>` |
| `texture_atlas_layout(...)` | build a sprite-sheet layout (feature `2d`) |
| `#[asset(image(sampler(filter = nearest)))]` | pixel-art sampling for this image |
| `standard_material` | wrap an image into a `StandardMaterial` (feature `3d`) |

### Also

- **Dynamic assets**: list paths in a `.assets.ron` file so artists change them without code.
- **Progress tracking** for a loading bar (`progress_tracking` example).
- **Failure state**: go somewhere if a file is missing.

### Links

[crates.io](https://crates.io/crates/bevy_asset_loader) · [docs.rs](https://docs.rs/bevy_asset_loader) · [repo](https://github.com/NiklasEi/bevy_asset_loader)

## bevy_common_assets

Load **RON, JSON, TOML, YAML, CSV, XML, MessagePack and more** straight into your own `serde` types as Bevy assets. No loader code: one plugin per file format. #assets

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_common_assets 0.18` supports **Bevy 0.20**.

```toml
bevy_common_assets = { version = "0.18", features = ["ron"] }   # pick your formats
serde = { version = "1", features = ["derive"] }
```

Write a [[Custom assets|custom asset loader]] once by hand to understand it; then use this.

### RON in five lines

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

### Why this is the best tuning tool

Combine with `file_watcher` ([[Hot reloading]]): edit the `.ron` file, save, and the numbers in the running game change. Put enemy stats, item tables, dialogue and level layouts in such files, not in Rust.

### Formats

| Feature | Plugin |
|:--|:--|
| `ron` | `RonAssetPlugin` |
| `json` | `JsonAssetPlugin` |
| `toml` | `TomlAssetPlugin` |
| `yaml` | `YamlAssetPlugin` |
| `csv` | `CsvAssetPlugin` |
| `xml`, `msgpack`, `postcard`, `cbor` | matching plugins |

### Links

[crates.io](https://crates.io/crates/bevy_common_assets) · [docs.rs](https://docs.rs/bevy_common_assets) · [repo](https://github.com/NiklasEi/bevy_common_assets)
