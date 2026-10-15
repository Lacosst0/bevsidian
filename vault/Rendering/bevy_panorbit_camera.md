---
lib: true
parent: Camera crates
icon: package
tags: [3d]
bevy: 0.19
crates: bevy_panorbit_camera=0.35
---
# bevy_panorbit_camera

An **orbit camera**: drag to rotate around a point, scroll to zoom, pan with another button. For model viewers, editors, strategy and building games. #3d

#0.19 · Bevy includes its own `PanOrbitCamera` (Cargo feature `pan_orbit_camera`, see [[Cameras]]), so you may not need this crate

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

- New project: the built-in controller.
- First-person: neither. Read the mouse yourself ([[Input#Keyboard and mouse|Input]]) or use the built-in free camera.
- 2D map panning: [[bevy_pancam]].

## Links

[crates.io](https://crates.io/crates/bevy_panorbit_camera) · [docs.rs](https://docs.rs/bevy_panorbit_camera) · [repo](https://github.com/Plonq/bevy_panorbit_camera)

## Next

[[Cameras]].
