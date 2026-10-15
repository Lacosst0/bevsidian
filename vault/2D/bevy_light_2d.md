---
lib: true
parent: 2D lighting crates
icon: package
tags: [2d]
crates: bevy_light_2d=0.11
---
# bevy_light_2d

**2D lighting with shadows**: darken the scene, add point and spot lights, and let walls block light. Turns flat sprites into moody dungeons. #2d

#0.20

```toml
bevy_light_2d = "0.11"
```

## A lit scene

Put `Light2d` on the camera (it sets the ambient light and enables the effect) and add light entities:

```rust
use bevy::prelude::*;
use bevy_light_2d::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Light2dPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Light2d::default()));

    commands.spawn((
        PointLight2d {
            intensity: 3.0,
            radius: 200.0,
            cast_shadows: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.6, 0.3, 0.3), Vec2::splat(60.0)),
        Transform::from_xyz(80.0, 0.0, 0.0),
    ));
}
```

Anything a camera sees is darkened by the ambient colour; lights lift it back up within their `radius`. Light follows its entity's `Transform`, so attach it to a player as a child ([[Relationships and hierarchy]]) for a torch.

## Shadows

Lights with `cast_shadows: true` are blocked by **occluders**. See the `occlusion` and `dungeon` examples for the occluder component and tile-based dungeons.

## Fits with

- [[Tilemaps]] for the walls the lights bounce off.
- [[Post-processing]] bloom for glowing light sources.
- Alternative: [[bevy_firefly]] (richer 2D lights, #0.19).

## Links

[crates.io](https://crates.io/crates/bevy_light_2d) · [docs.rs](https://docs.rs/bevy_light_2d) · [repo](https://github.com/jgayfer/bevy_light_2d)

## Next

[[2D]].
