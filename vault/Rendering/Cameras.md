---
icon: camera
tags: [rendering, beginner]
---
# Cameras

A camera is an entity. Where it is and what component it has decides what you see. #rendering #beginner

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

Bevy 0.20 includes camera controllers (behind Cargo features); add the plugin and put the component on a camera:

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

## Screen shake, follow, bloom

Cameras are entities: shake by changing `Transform`, follow by `smooth_nudge` ([[Transforms]]), glow by adding `Bloom` ([[Post-processing]]).

## Next

[[Colors and math]].
