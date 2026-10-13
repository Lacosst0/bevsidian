---
icon: move
tags: [rendering, beginner]
---
# Transforms

Where an entity is, which way it faces, how big it is. Every visible thing has a [`Transform`]. #rendering #beginner

## The three parts

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::splat(50.0)),
        Transform {
            translation: Vec3::new(100.0, 50.0, 0.0),   // position
            rotation: Quat::from_rotation_z(0.5),       // radians, counter-clockwise
            scale: Vec3::new(2.0, 1.0, 1.0),            // stretch
        },
    ));

    // Shortcuts:
    commands.spawn(Transform::from_xyz(0.0, 3.0, 0.0));
    commands.spawn(Transform::from_scale(Vec3::splat(2.0)));
    commands.spawn(Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y));
}
```

## Axes

| | 2D | 3D |
|:--|:--|:--|
| **x** | right | right |
| **y** | up | up |
| **z** | *draw order*: bigger is in front | towards the viewer |

Bevy is **y-up**, right-handed, and the 2D origin is the centre of the screen. In 2D, give overlapping things different `z` (0, 1, 2...) so the order is predictable.

## Moving things

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player;

fn movement(time: Res<Time>, mut player: Single<&mut Transform, With<Player>>) {
    player.translation.x += 100.0 * time.delta_secs();      // 100 units/second right
    player.rotate_z(1.0 * time.delta_secs());               // turn
    player.scale = Vec3::splat(1.0 + time.elapsed_secs().sin() * 0.2);   // pulse
}

fn face(mut player: Single<&mut Transform, With<Player>>) {
    player.look_at(Vec3::ZERO, Vec3::Y);                    // 3D: aim at a point
    let forward = player.forward();                          // direction it faces
    player.translation += forward * 0.1;
}
```

Multiply by `delta_secs()` so speed does not depend on frame rate ([[Time and timers]]).

## Parents and children

A child's `Transform` is **relative to its parent** ([[Relationships and hierarchy]]). Move a car and its wheels follow.

[`GlobalTransform`] holds the final world position. Bevy computes it in `PostUpdate`. Read it when you need "where is this on screen?" but **write** only `Transform`.

```rust
use bevy::prelude::*;

fn wheels(mut commands: Commands) {
    commands.spawn((
        Name::new("Car"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        children![(Name::new("Wheel"), Transform::from_xyz(1.0, -0.5, 0.0))],
    ));
}

fn where_is_it(query: Query<(&Name, &GlobalTransform)>) {
    for (name, global) in &query {
        info!("{name} is at {}", global.translation());
    }
}
```

## Smooth following

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player;

fn camera_follow(
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let target = player.translation.with_z(camera.translation.z);
    // Frame-rate independent smoothing: higher decay = snappier.
    camera.translation.smooth_nudge(&target, 5.0, time.delta_secs());
}
```

## Common mistakes

- Setting `Transform` on a child expecting world coordinates. It is local.
- Two queries on `&mut Transform` in one system: add `Without<...>` filters ([[Advanced queries]]).
- Forgetting `z` in 2D when sprites overlap.
- Moving a physics body by writing its `Transform`: let the physics engine do it ([[Physics]]).

## Next

[[Cameras]].
