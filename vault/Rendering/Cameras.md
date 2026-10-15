---
icon: camera
tags: [rendering]
---
# Cameras

A camera is an entity. Where it is and what component it has decides what you see. #rendering

## 2D and 3D

```rust
use bevy::prelude::*;

fn setup_2d(mut commands: Commands) {
    commands.spawn(Camera2d);       // looks along -z; 1 unit = 1 pixel at 100% zoom
}

fn setup_3d(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

Both add the camera, a projection and a transform for you ([[Component lifecycle|required components]]).

## Background colour

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.12)))
        .run();
}
```

Or per camera: `Camera { clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }`.

## Zoom (2D)

2D cameras use an **orthographic** projection; change its `scale` (bigger shows more):

```rust
use bevy::prelude::*;

fn zoom(mut projection: Single<&mut Projection, With<Camera2d>>) {
    if let Projection::Orthographic(ortho) = &mut **projection {
        ortho.scale = 0.5;   // 2x zoom in
    }
}
```

## Field of view (3D)

```rust
use bevy::prelude::*;

fn lens(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            ..default()
        }),
    ));
}
```

## Ready-made controllers

Bevy includes camera controllers (behind Cargo features); add the plugin and put the component on a camera:

| Controller | For | Plugin / component |
|:--|:--|:--|
| Free camera | fly around, debugging | `FreeCameraPlugin`, `FreeCamera` |
| Pan-orbit | orbit around an object, editors | `DefaultPanOrbitCameraPlugins`, `PanOrbitCamera` |
| Pan | top-down maps, 2D | `PanCameraPlugin`, `PanCamera` |

```rust
use bevy::prelude::*;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FreeCameraPlugin))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((Camera3d::default(), FreeCamera::default()));
        })
        .run();
}
```

Enable with `features = ["free_camera"]` (and `pan_camera`, `pan_orbit_camera` for the others). Find all the options in the `bevy::camera_controller` module.

## More than one camera

Spawn several; give each a different `order` and a `viewport` for split screen, or a render target to draw into a texture (mirrors, minimap):

```rust
use bevy::prelude::*;

fn minimap(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,                      // drawn after camera 0
            clear_color: ClearColorConfig::None,  // keep what is under it
            ..default()
        },
    ));
}
```

Use `RenderLayers` to decide which cameras see which entities.

## Screen to world and back

```rust
use bevy::prelude::*;

fn cursor_world(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let (camera, camera_transform) = *camera;
    if let Some(position) = window.cursor_position()
        && let Ok(world) = camera.viewport_to_world_2d(camera_transform, position)
    {
        info!("cursor in the world: {world}");
    }
}
```

## A camera that follows

Do not copy the player's position: **ease towards it**. `smooth_nudge` moves a fraction of the gap each frame, correctly for any frame rate:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Player;

fn follow(
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let target = player.translation.truncate().extend(camera.translation.z);
    camera.translation.smooth_nudge(&target, 5.0, time.delta_secs());
}
```

The `5.0` is how fast it catches up (higher is snappier). Put it in `PostUpdate`, after the player's transform changes, or the camera lags a frame behind and the player seems to jitter.

## Keep the camera inside the level

Clamp the follow target so the camera never shows outside the map:

```rust
use bevy::prelude::*;

/// The camera centre limited so a view of `view` size stays inside `bounds`.
fn clamp_to_level(centre: Vec2, view: Vec2, bounds: Rect) -> Vec2 {
    let half = view / 2.0;
    Vec2::new(
        centre.x.clamp(bounds.min.x + half.x, (bounds.max.x - half.x).max(bounds.min.x + half.x)),
        centre.y.clamp(bounds.min.y + half.y, (bounds.max.y - half.y).max(bounds.min.y + half.y)),
    )
}
```

(The `max` keeps it sane when the level is smaller than the screen.)

## Pixel-art games

Pixel art needs the screen to be a whole multiple of the art's pixels, or edges shimmer. Two parts: sample images without smoothing (`ImagePlugin::default_nearest()`), and use a **fixed-height** view that scales with the window:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera2d,
                Projection::Orthographic(OrthographicProjection {
                    scaling_mode: bevy::camera::ScalingMode::FixedVertical { viewport_height: 180.0 },
                    ..OrthographicProjection::default_2d()
                }),
            ));
        })
        .run();
}
```

`FixedVertical` always shows 180 world units top to bottom: a wider window shows more to the sides, not a bigger picture. `AutoMin`/`AutoMax` and `Fixed` are the other scaling modes.

## Which camera sees what

Each camera draws the entities whose `RenderLayers` overlap with its own. The default is layer 0 for both, so everything is seen by everyone. Put the minimap-only icons on layer 1 and give the minimap camera layer 1 too:

```rust
use bevy::{camera::visibility::RenderLayers, prelude::*};

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, RenderLayers::layer(0)));
    commands.spawn((
        Camera2d,
        Camera { order: 1, ..default() },
        RenderLayers::from_layers(&[0, 1]),   // sees the world plus the minimap icons
    ));
    commands.spawn((Sprite::default(), RenderLayers::layer(1)));  // only the second camera sees this
}
```

## Screen shake, follow, bloom

Cameras are entities: shake by changing `Transform` ([[Game feel]]), glow by adding `Bloom` ([[Post-processing]]).

## Next

[[Colors and math]].

Crates that add camera controls, and when you still need them: [[Camera crates]].
