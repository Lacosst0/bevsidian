---
icon: package
tags: [physics]
bevy: 0.19
crates: avian2d=0.7; bevy_transform_interpolation=0.5
---
# Avian

**Avian** is a 2D and 3D physics engine **made for Bevy's ECS**: no wrapped foreign engine, everything is components and systems you can read, query and extend. Crates `avian2d` and `avian3d`. (Formerly "bevy_xpbd".) #physics

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.7` targets **Bevy 0.19**; `main` is still on 0.19. For 0.20 today, use [[bevy_rapier]]. The code below compiles against 0.19 + `avian2d 0.7`.

```toml
avian2d = "0.7"      # or avian3d = "0.7"
```

## Why pick it

- Feels like Bevy: forces, velocities and mass are components; collisions are observable; the physics schedule is just schedules ([[Schedules and sets]]).
- Rigid bodies (dynamic, kinematic, static), colliders with layers and materials, joints, ray/shape casts via `SpatialQuery`, continuous collision detection, sleeping.
- Built-in **transform interpolation** for fixed-timestep physics ([[Avian#bevy_transform_interpolation|bevy_transform_interpolation]] exists for engines without it).
- Debug rendering of colliders, contacts and joints.

## Bodies and colliders

```rust
use avian2d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec2::NEG_Y * 600.0))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // A fixed floor.
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(500.0, 20.0),
        Sprite::from_color(Color::srgb(0.3, 0.3, 0.35), Vec2::new(500.0, 20.0)),
        Transform::from_xyz(0.0, -150.0, 0.0),
    ));

    // A bouncy ball.
    commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(20.0),
        Restitution::new(0.8),
        LinearVelocity(Vec2::new(80.0, 0.0)),
        Sprite::from_color(Color::srgb(0.9, 0.4, 0.4), Vec2::splat(40.0)),
        Transform::from_xyz(-100.0, 150.0, 0.0),
    ));
}
```

`RigidBody` + `Collider` is the minimum. Everything else (`Mass`, `Friction`, `LinearVelocity`, `GravityScale`, `LockedAxes`) is optional.

## Move things by physics

```rust
use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)] struct Player;

fn walk(keys: Res<ButtonInput<KeyCode>>, mut player: Single<&mut LinearVelocity, With<Player>>) {
    let x = keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8;
    player.x = x as f32 * 200.0;
}
```

## Avian or Rapier?

| | Avian | [[bevy_rapier]] |
|:--|:--|:--|
| Written | for Bevy's ECS | as a standalone engine, then wrapped |
| Hackability | high: plain components and schedules | lower, but a huge feature list |
| Bevy 0.20 | pending | **released** |
| Docs | docs.rs and examples | docs.rs, rapier.rs guide |

Both are good. Choose by the Bevy version you need and the features you use.

## Links

[crates.io (2D)](https://crates.io/crates/avian2d) · [crates.io (3D)](https://crates.io/crates/avian3d) · [docs.rs](https://docs.rs/avian2d) · [repo](https://github.com/avianphysics/avian)

## bevy_transform_interpolation

When game logic runs at a **fixed rate** ([[Time and timers]]) but the screen refreshes faster, moving things look jittery. This crate **interpolates** (or extrapolates) `Transform` between fixed steps so motion stays smooth. #physics

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.5` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_transform_interpolation = "0.5"
```

### Turn it on per entity

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

### When you need it

- You run physics or movement in `FixedUpdate` ([[bevy_rapier]], [[Avian]], your own, see [[Physics]]).
- It looks stuttery on a high-refresh-rate monitor.
- **Not** when logic runs in `Update`: movement there is already per frame.

Interpolation shows the world slightly in the past; *extrapolation* guesses the future and may overshoot. The crate supports both.

### Links

[crates.io](https://crates.io/crates/bevy_transform_interpolation) · [docs.rs](https://docs.rs/bevy_transform_interpolation) · [repo](https://github.com/Jondolf/bevy_transform_interpolation)

## Next

[[Physics]].
