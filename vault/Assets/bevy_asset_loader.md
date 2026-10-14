---
icon: package
tags: [assets]
crates: bevy_asset_loader={ version = "0.28", features = ["2d"] }
---
# bevy_asset_loader

Describe all your assets **in a struct**; the crate loads them during a loading [[States|state]], then moves on and gives you the finished resource. Replaces hand-written preloading and `is_loaded` checks ([[Assets]]). #assets

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_asset_loader 0.28` supports **Bevy 0.20**.

```toml
bevy_asset_loader = { version = "0.28", features = ["2d"] }   # "2d" adds texture atlas support, "3d" materials
```

## An asset collection

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

## Useful attributes

| Attribute | Does |
|:--|:--|
| `#[asset(path = "...")]` | one file |
| `#[asset(paths("a.png", "b.png"), collection)]` | a list |
| `collection(typed, mapped)` | a folder as `HashMap<String, Handle<T>>` |
| `texture_atlas_layout(...)` | build a sprite-sheet layout (feature `2d`) |
| `#[asset(image(sampler(filter = nearest)))]` | pixel-art sampling for this image |
| `standard_material` | wrap an image into a `StandardMaterial` (feature `3d`) |

## Also

- **Dynamic assets**: list paths in a `.assets.ron` file so artists change them without code.
- **Progress tracking** for a loading bar (`progress_tracking` example).
- **Failure state**: go somewhere if a file is missing.

## Links

[crates.io](https://crates.io/crates/bevy_asset_loader) · [docs.rs](https://docs.rs/bevy_asset_loader) · [repo](https://github.com/NiklasEi/bevy_asset_loader)

## Next

[[Assets]], [[bevy_common_assets]].
