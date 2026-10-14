---
icon: package
tags: [physics]
bevy: 0.19
crates: bevy_transform_interpolation=0.5
---
# bevy_transform_interpolation

When game logic runs at a **fixed rate** ([[Time and timers]]) but the screen refreshes faster, moving things look jittery. This crate **interpolates** (or extrapolates) `Transform` between fixed steps so motion stays smooth. #physics

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.5` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_transform_interpolation = "0.5"
```

## Turn it on per entity

```rust
use bevy::prelude::*;
use bevy_transform_interpolation::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TransformInterpolationPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, spin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.4, 0.8, 0.5), Vec2::splat(50.0)),
        Transform::default(),
        TransformInterpolation,          // smooth translation, rotation and scale
    ));
}

fn spin(mut query: Query<&mut Transform, With<TransformInterpolation>>) {
    for mut transform in &mut query {
        transform.rotate_z(0.05);        // moved once per fixed step...
        transform.translation.x += 1.0;  // ...but drawn smoothly every frame
    }
}
```

Use `TransformInterpolationPlugin::interpolate_all()` to smooth every entity, or just `TranslationInterpolation`, `RotationInterpolation`, `ScaleInterpolation` for parts.

## When you need it

- You run physics or movement in `FixedUpdate` ([[bevy_rapier]], [[Avian]], your own, see [[Physics]]).
- It looks stuttery on a high-refresh-rate monitor.
- **Not** when logic runs in `Update`: movement there is already per frame.

Interpolation shows the world slightly in the past; *extrapolation* guesses the future and may overshoot. The crate supports both.

## Links

[crates.io](https://crates.io/crates/bevy_transform_interpolation) · [docs.rs](https://docs.rs/bevy_transform_interpolation) · [repo](https://github.com/Jondolf/bevy_transform_interpolation)

## Next

[[Physics]].
