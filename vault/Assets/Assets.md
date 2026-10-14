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

`load` returns **immediately**. The file is read on another thread; a sprite with an unfinished handle simply isn't drawn until the data arrives. Loading the same path twice gives the same asset, not a copy.

## Handles and the Assets store

- The one real copy of every image lives in the `Assets<Image>` resource.
- A [`Handle<Image>`] is a small reference to it. `Sprite`, `Mesh3d`, `AudioPlayer` all store handles.
- Handles are reference counted. When the last handle is dropped, the asset is unloaded.

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

Swapping the **handle** changes one entity. Editing the **asset** (`images.get_mut(&handle)`) changes every entity sharing it.

## Preload what you need

If you load an asset in a system and drop the handle, it unloads, then reloads next time (the enemy may flicker in). Keep the handles in a resource:

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

Keep a [[States|state]] while assets load, and move on when they are ready:

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

`is_loaded_with_dependencies` also waits for things the asset needs (a model's textures). For many assets use [[Asset crates#bevy_asset_loader|bevy_asset_loader]].

## Progress, and what to do when a load fails

`is_loaded_with_dependencies` gives yes or no. For a progress bar, ask each handle for its `LoadState` and count:

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

A load fails when the file is missing, misspelled (paths are case sensitive on Linux and the web, so `Player.png` works on your laptop and breaks in release) or in a format whose Cargo feature is off. Bevy logs an error with the path; read the console.

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

`load_folder` does not work in the browser, which cannot list a directory. On the web, list the files yourself in a manifest ([[Custom assets]]).

## Reacting to changes

Every asset store sends an `AssetEvent` when something is added, modified or removed. This is how a system notices a hot-reloaded file ([[Hot reloading]]) or a texture that just finished loading:

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

Not every asset is a file. Meshes, materials, colour textures and animation clips are often made in code and added to the store; the handle works exactly like a loaded one:

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

For a single-file game (a jam entry, a tool) embed small assets in the executable with the `embedded_asset!` macro and load them with an `embedded://` path. For everything else keep the `assets/` folder next to the game: the files can then be updated without recompiling. See [[Release builds]] for what to copy when you ship.

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

Status for **Bevy 0.20**, checked 2026-10-13: **released** (a crates.io release uses 0.20) · **git** (the `main` branch is on 0.20; use a git dependency) · **pending** (the latest release targets an older Bevy) · **built in** (Bevy has it now). Check crates.io before you start.

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[Asset crates#bevy_asset_loader\|bevy_asset_loader]] | declare assets in a struct, loading states | released `0.28` |
| [[Asset crates#bevy_common_assets\|bevy_common_assets]] | RON / JSON / TOML... as assets | released `0.18` |
| [[bevy_pkv]] | key-value saves, desktop and web | pending `0.16` |

## Next

[[Saving and loading]] to keep data between runs, [[Custom assets]] to write your own loader, or [[Scenes and BSN]].
