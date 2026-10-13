---
icon: package
tags: [assets, beginner]
---
# Assets

Images, sounds, models, fonts, scenes: anything that lives in a file. The [`AssetServer`] loads them in the background and hands you a **handle**. #assets #beginner

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

`is_loaded_with_dependencies` also waits for things the asset needs (a model's textures). For many assets use `bevy_asset_loader` ([[Libraries]]).

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

## Next

[[Scenes and BSN]], or write your own loader in [[Custom assets]].
