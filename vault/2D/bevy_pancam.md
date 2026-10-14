---
icon: package
tags: [2d]
bevy: 0.19
crates: bevy_pancam=0.21
---
# bevy_pancam

A **pan and zoom camera for 2D**: drag the map, scroll to zoom (towards the cursor), WASD/arrows to move. For map editors, strategy and simulation games. #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.21` targets **Bevy 0.19**. Bevy 0.20 includes a built-in `PanCamera` controller (feature `pan_camera`, see [[Cameras]]). Compiled against 0.19.

```toml
bevy_pancam = "0.21"
```

## Usage

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

## Links

[crates.io](https://crates.io/crates/bevy_pancam) · [docs.rs](https://docs.rs/bevy_pancam) · [repo](https://github.com/johanhelsing/bevy_pancam)

## Next

[[Cameras]].
