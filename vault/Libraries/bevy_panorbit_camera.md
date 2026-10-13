---
icon: package
tags: [lib, 3d]
bevy: 0.19
crates: bevy_panorbit_camera=0.35
---
# bevy_panorbit_camera

An **orbit camera**: drag to rotate around a point, scroll to zoom, pan with another button. For model viewers, editors, strategy and building games. #lib #3d

> [!NOTE]
> **Bevy 0.20 includes its own.** `bevy::camera_controller::pan_orbit_camera` (Cargo feature `pan_orbit_camera`) provides a `PanOrbitCamera` that works the same way, so on Bevy 0.20 you do not need this crate: see [[Cameras]]. The latest release `0.35` of this crate targets **Bevy 0.19**; the code below is compiled against 0.19.

```toml
bevy_panorbit_camera = "0.35"
```

## Usage

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

## Choosing

- New project on 0.20: the built-in controller.
- First-person: neither. Read the mouse yourself ([[Keyboard and mouse]]) or use the built-in free camera.
- 2D map panning: [[bevy_pancam]].

## Links

[crates.io](https://crates.io/crates/bevy_panorbit_camera) · [docs.rs](https://docs.rs/bevy_panorbit_camera) · [repo](https://github.com/Plonq/bevy_panorbit_camera)

## Next

[[Cameras]], [[Libraries]].
