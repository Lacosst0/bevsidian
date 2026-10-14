---
lib: true
icon: package
tags: [2d, 3d]
bevy: 0.19
crates: bevy_pancam=0.21; bevy_panorbit_camera=0.35
---
# Camera crates

Ready-made camera controls. Bevy 0.20 now ships its own pan and orbit controllers ([[Cameras]]), so these are mostly for older versions.

## bevy_pancam

A **pan and zoom camera for 2D**: drag the map, scroll to zoom (towards the cursor), WASD/arrows to move. For map editors, strategy and simulation games. #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.21` targets **Bevy 0.19**. Bevy 0.20 includes a built-in `PanCamera` controller (feature `pan_camera`, see [[Cameras]]). Compiled against 0.19.

```toml
bevy_pancam = "0.21"
```

### Usage

```rust
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin::default()))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera2d,
                PanCam {
                    min_scale: 0.5,          // zoom limits
                    max_scale: 10.0,
                    ..default()
                },
            ));
        })
        .run();
}
```

Options include which mouse buttons drag, bounding box limits (`min_x`..`max_y`), `zoom_to_cursor`, and enabling/disabling it at runtime (for when the pointer is over UI).

### Links

[crates.io](https://crates.io/crates/bevy_pancam) · [docs.rs](https://docs.rs/bevy_pancam) · [repo](https://github.com/johanhelsing/bevy_pancam)

## bevy_panorbit_camera

An **orbit camera**: drag to rotate around a point, scroll to zoom, pan with another button. For model viewers, editors, strategy and building games. #3d

> [!NOTE]
> **Bevy 0.20 includes its own.** `bevy::camera_controller::pan_orbit_camera` (Cargo feature `pan_orbit_camera`) provides a `PanOrbitCamera` that works the same way, so on Bevy 0.20 you do not need this crate: see [[Cameras]]. The latest release `0.35` of this crate targets **Bevy 0.19**; the code below is compiled against 0.19.

```toml
bevy_panorbit_camera = "0.35"
```

### Usage

```rust
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera::default(),     // adds a Camera3d via required components
    ));
}
```

Default controls: left mouse rotates, right mouse pans, wheel zooms; touch and trackpad gestures are supported. Fields on `PanOrbitCamera` change focus point, limits, sensitivity and buttons.

### Choosing

- New project on 0.20: the built-in controller.
- First-person: neither. Read the mouse yourself ([[Keyboard and mouse]]) or use the built-in free camera.
- 2D map panning: [[Camera crates#bevy_pancam|bevy_pancam]].

### Links

[crates.io](https://crates.io/crates/bevy_panorbit_camera) · [docs.rs](https://docs.rs/bevy_panorbit_camera) · [repo](https://github.com/Plonq/bevy_panorbit_camera)
