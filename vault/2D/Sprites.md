---
icon: image
tags: [2d]
---
# Sprites

A [`Sprite`] is a 2D picture or a coloured rectangle. Spawn one with a `Transform` and a camera sees it. #2d

## From an image, or a colour

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(asset_server.load("hero.png")));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.3, 0.3), Vec2::new(100.0, 40.0)),
        Transform::from_xyz(0.0, -80.0, 0.0),
    ));
}
```

`from_color` is perfect for prototypes: no art needed yet.

## Order: z decides

Run it and see: the red square was spawned *first* but is drawn *in front*, because its `z` is bigger. Click the stage to add squares.

```rust example=sprites
use bevy::prelude::*;

#[derive(Component)]
struct Breathing;

fn spawn_squares(mut commands: Commands) {
    // Later spawns are not drawn on top: z decides.
    commands.spawn((
        Sprite::from_color(Color::srgb(0.95, 0.45, 0.4), Vec2::splat(60.0)),
        Transform::from_xyz(-30.0, 0.0, 1.0),
    ));
    commands.spawn((
        Breathing,
        Sprite::from_color(Color::srgb(0.35, 0.75, 0.9), Vec2::splat(60.0)),
        Transform::from_xyz(0.0, 10.0, 0.0),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.98, 0.75, 0.3), Vec2::new(90.0, 20.0)),
        Transform::from_xyz(40.0, -30.0, 2.0),
    ));
}
```

Give each *layer* its own z: background -10, ground 0, characters 1, effects 5.

## Change it while playing

A sprite is data; change it in a system. This one makes a square breathe by changing `scale`:

```rust example=sprites
fn breathe(time: Res<Time>, mut squares: Query<&mut Transform, With<Breathing>>) {
    for mut transform in &mut squares {
        let scale = 1.0 + 0.15 * (time.elapsed_secs() * 3.0).sin();
        transform.scale = Vec3::splat(scale);
    }
}
```

## Click to place

Pointer events ([[Picking]]) say where the click landed. This observer adds a square there, with a pop sound (yours, if you made one on [[Audio]]):

```rust example=sprites
#[derive(Component)]
struct Sky;

#[derive(Resource)]
struct Sounds {
    pop: Handle<AudioSource>,
}

fn place_square(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    sounds: Res<Sounds>,
    mut commands: Commands,
) {
    // Only clicks on the background, not on other squares.
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    commands.spawn((AudioPlayer::new(sounds.pop.clone()), PlaybackSettings::DESPAWN));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.6, 0.5, 0.95), Vec2::splat(24.0)),
        Transform::from_translation(at.truncate().extend(3.0)),
    ));
}
```

## The Sprite fields

| Field | Does |
|:--|:--|
| `image` | the picture |
| `color` | tint, multiplied with the image (`Color::WHITE` = unchanged) |
| `custom_size` | draw at this size instead of the image's own |
| `flip_x`, `flip_y` | mirror (face left with `flip_x = true`) |
| `rect` | draw only part of the image ([[Texture atlases]]) |
| `texture_atlas` | pick a frame from a sheet |
| `image_mode` | `Auto`, `Sliced` (9-slice panels), `Tiled` (repeat) |

```rust
use bevy::prelude::*;

fn variants(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Tinted and mirrored
    commands.spawn(Sprite {
        flip_x: true,
        color: Color::srgb(1.0, 0.6, 0.6),
        ..Sprite::from_image(asset_server.load("hero.png"))
    });

    // A repeating floor
    commands.spawn(Sprite {
        custom_size: Some(Vec2::new(640.0, 32.0)),
        image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y: false, stretch_value: 1.0 },
        ..Sprite::from_image(asset_server.load("floor.png"))
    });
}
```

## Anchor: where is the middle?

By default the transform is the sprite's centre. The `Anchor` component moves it, e.g. to stand a character on the ground by its feet:

```rust
use bevy::{prelude::*, sprite::Anchor};

fn feet(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("hero.png")),
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(0.0, -100.0, 1.0),   // the feet are at y = -100
    ));
}
```

## Pixel art

`DefaultPlugins.set(ImagePlugin::default_nearest())` stops Bevy from smoothing images when it scales them. Scale your sprites by whole numbers (`Transform::from_scale(Vec3::splat(4.0))`) or use a camera with a fixed pixel size.

## Make it yours

Press the **brush** button above a game preview and draw a 16 × 16 sprite. It's saved, and every example uses it: the ship in Dodge ([[Make a game]]), [[Transforms]] and [[Input]], the player in [[Game AI]] and [[Game feel]], the agent in [[Pathfinding]], the balls in [[Physics]]. Until you do, they fly the starting pixel ship. Build a voxel model for the rocks too: [[3D]].

## Next

[[Texture atlases]], then [[Sprite animation]].
