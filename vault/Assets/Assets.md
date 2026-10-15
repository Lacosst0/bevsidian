---
icon: package
tags: [assets]
---
# Assets

Images, sounds, models, fonts, scenes: anything that lives in a file. The [`AssetServer`] loads them in the background and hands you a **handle**. #assets

## Load something

Put files in an `assets/` folder next to `Cargo.toml`, then:

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let image: Handle<Image> = asset_server.load("player.png");   // reads assets/player.png
    commands.spawn(Sprite::from_image(image));
}
```

`load` returns **at once**; the file is read in the background, and the sprite appears when it's ready. Loading a path twice gives the same asset, not a copy.

## Handles and the Assets store

The real image lives once in the `Assets<Image>` resource; a [`Handle<Image>`] is a cheap, reference-counted pointer to it. `Sprite`, `Mesh3d` and `AudioPlayer` all hold handles, and when the last handle goes the asset is unloaded.

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player;

fn change_costume(
    mut sprite: Single<&mut Sprite, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    // Point this one sprite at a different image.
    sprite.image = asset_server.load("player_red.png");
}

fn make_image(mut images: ResMut<Assets<Image>>) -> Handle<Image> {
    // Create an asset from code, and get its handle.
    images.add(Image::default())
}
```

> [!TIP]
> Swap the **handle** to change one entity. Edit the **asset** (`images.get_mut(&handle)`) to change every entity that shares it.

## Preload what you need

Drop the last handle and the asset unloads, then reloads next time you ask (the enemy flickers in). Keep handles in a resource:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct GameAssets {
    enemy: Handle<Image>,
    music: Handle<AudioSource>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        enemy: asset_server.load("enemy.png"),
        music: asset_server.load("theme.ogg"),
    });
}

fn spawn_enemy(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn(Sprite::from_image(assets.enemy.clone()));
}
```

## A loading screen

Stay in a loading [[States|state]] until everything is in:

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Screen { #[default] Loading, Playing }

#[derive(Resource)]
struct GameAssets { enemy: Handle<Image> }

fn check_loaded(
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<Screen>>,
) {
    if asset_server.is_loaded_with_dependencies(&assets.enemy) {
        next.set(Screen::Playing);
    }
}

fn main() {
    App::new()
        .init_state::<Screen>()
        .add_systems(Update, check_loaded.run_if(in_state(Screen::Loading)))
        .run();
}
```

`is_loaded_with_dependencies` also waits for what the asset needs (a model's textures). Many assets? [[bevy_asset_loader]] does this for you.

## Progress, and what to do when a load fails

For a progress bar, count each handle's `LoadState`:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct Loading(Vec<UntypedHandle>);

/// Loaded handles out of all, as 0..1. Logs the ones that failed.
fn progress(loading: &Loading, asset_server: &AssetServer) -> f32 {
    let mut done = 0;
    for handle in &loading.0 {
        match asset_server.load_state(handle.id()) {
            bevy::asset::LoadState::Loaded => done += 1,
            bevy::asset::LoadState::Failed(error) => {
                error!("could not load {:?}: {error}", handle.path());
                done += 1;      // do not wait forever for a file that is not coming
            }
            _ => {}
        }
    }
    done as f32 / loading.0.len().max(1) as f32
}

fn start(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Loading(vec![
        asset_server.load::<Image>("enemy.png").untyped(),
        asset_server.load::<AudioSource>("theme.ogg").untyped(),
    ]));
}
```

> [!WARNING]
> Loads fail on a missing file, a format whose Cargo feature is off, or a wrong case: paths are case-sensitive on Linux and the web, so `Player.png` works on your laptop and breaks in release. Bevy logs the path.

## A whole folder

Load every file in a folder at once and keep the one handle:

```rust
use bevy::{asset::LoadedFolder, prelude::*};

#[derive(Resource)]
struct Sprites(Handle<LoadedFolder>);

fn load_folder(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sprites(asset_server.load_folder("sprites")));
}

fn count(sprites: Res<Sprites>, folders: Res<Assets<LoadedFolder>>) {
    if let Some(folder) = folders.get(&sprites.0) {
        info!("{} files in sprites/", folder.handles.len());
    }
}
```

Browsers can't list directories, so on the web keep a manifest of files instead ([[Custom assets]]).

## Reacting to changes

Each asset store sends an `AssetEvent` when an asset is added, modified or removed: how you notice a finished load or a hot-reloaded file ([[Hot reloading]]):

```rust
use bevy::prelude::*;

fn on_image_events(mut events: MessageReader<AssetEvent<Image>>, images: Res<Assets<Image>>) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                if let Some(image) = images.get(*id) {
                    info!("image ready: {}x{}", image.width(), image.height());
                }
            }
            AssetEvent::Modified { id } => info!("image {id} changed"),
            AssetEvent::Removed { id } => info!("image {id} gone"),
            _ => {}
        }
    }
}
```

## Build assets in code

Meshes, materials and textures are often made in code. Add them to their store and the handle works like a loaded one:

```rust
use bevy::prelude::*;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(40.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.9, 0.4, 0.4))),
    ));
}
```

## Ship the assets inside the binary

For a single-file game, embed small assets with `embedded_asset!` and load them from `embedded://`. Otherwise ship the `assets/` folder beside the game, so files can change without a rebuild ([[Release builds]]).

## Paths and settings

| Want | Do |
|:--|:--|
| a sub-folder | `load("sprites/hero.png")` |
| a part of a file | `load("model.gltf#Scene0")` (glTF labels) |
| a different root folder | `AssetPlugin { file_path: "data".into(), ..default() }` |
| nearest-neighbour pixel art | `ImagePlugin::default_nearest()` in `DefaultPlugins.set(...)` |
| reload on save | feature `file_watcher`, or `bevy/dev` ([[Hot reloading]]) |
| load from the web | feature `http` / `https`, path `https://...` |

## Formats

Out of the box: `.png`, `.ogg` (Vorbis), `.gltf`/`.glb`, `.wgsl`/`.wesl` shaders. Others (`.jpg`, `.mp3`, `.wav`, `.ktx2`...) are Cargo features: see [[Project setup]] and [[Audio]].

## Libraries

Crates compared side by side: [[Asset crates]].

| Crate | For | Bevy |
|:--|:--|:--|
| [[bevy_asset_loader]] | declare assets in a struct, loading states | #0.20 |
| [[bevy_common_assets]] | RON / JSON / TOML... as assets | #0.20 |
| [[bevy_pkv]] | key-value saves, desktop and web | #0.19 |

## Next

[[Saving and loading]] to keep data between runs, [[Custom assets]] to write your own loader, or [[Scenes and BSN]].
