---
icon: move
tags: [rendering]
---
# Transforms

Where a thing is, which way it faces, how big it is. Everything you can see has a [`Transform`]. #rendering

## The three parts

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::splat(50.0)),
        Transform {
            translation: Vec3::new(100.0, 50.0, 0.0), // position
            rotation: Quat::from_rotation_z(0.5),     // radians, counter-clockwise
            scale: Vec3::new(2.0, 1.0, 1.0),          // stretch
        },
    ));

    // Shortcuts.
    commands.spawn(Transform::from_xyz(0.0, 3.0, 0.0));
    commands.spawn(Transform::from_scale(Vec3::splat(2.0)));
    commands.spawn(Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y));
}
```

| Axis | 2D | 3D |
|:--|:--|:--|
| **x** | right | right |
| **y** | up | up |
| **z** | draw order: bigger is in front | toward the viewer |

Bevy is **y-up** and right-handed, and in 2D the origin is the middle of the screen. Give overlapping sprites different `z` so the order is predictable.

## Move and turn

Click anywhere: the ship flies there and turns to face where it's going. `fly` changes `translation`, `turn` changes `rotation`.

```rust example=movement
use bevy::prelude::*;

#[derive(Component)]
struct Ship;

/// Where the ship is going.
#[derive(Component)]
struct Target(Vec2);

fn fly(time: Res<Time>, mut ships: Query<(&mut Transform, &Target)>) {
    const SPEED: f32 = 90.0;
    for (mut transform, target) in &mut ships {
        let to_target = target.0 - transform.translation.truncate();
        // Never overshoot: move at most the distance that is left.
        let step = (SPEED * time.delta_secs()).min(to_target.length());
        transform.translation += (to_target.normalize_or_zero() * step).extend(0.0);
    }
}

fn turn(mut ships: Query<(&mut Transform, &Target)>) {
    for (mut transform, target) in &mut ships {
        let to_target = target.0 - transform.translation.truncate();
        if to_target.length() > 1.0 {
            // The nose points up (+y): turn a quarter less than the direction.
            let angle = to_target.to_angle() - std::f32::consts::FRAC_PI_2;
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}
```

Speeds are multiplied by `delta_secs()`, so the ship flies just as fast at 30 or 144 fps ([[Time and timers]]). Draw your own ship with the brush button and it flies that instead. The click handling is in [[Input]].

Handy helpers:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Player;

fn helpers(time: Res<Time>, mut player: Single<&mut Transform, With<Player>>) {
    player.rotate_z(1.0 * time.delta_secs());                          // spin
    player.scale = Vec3::splat(1.0 + time.elapsed_secs().sin() * 0.2); // pulse
    player.look_at(Vec3::ZERO, Vec3::Y);                               // 3D: aim at a point
    let forward = player.forward();                                    // the way it faces
    player.translation += forward * 0.1;
}
```

## Parents and children

A child's `Transform` is **relative to its parent** ([[Relationships and hierarchy]]): move the car and the wheels come along. [`GlobalTransform`] is the final position in the world, computed in `PostUpdate`. Read it, but only ever write `Transform`.

```rust
use bevy::prelude::*;

fn car(mut commands: Commands) {
    commands.spawn((
        Name::new("Car"),
        Transform::default(),
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

`smooth_nudge` eases toward a target at the same speed on every frame rate:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Player;

fn camera_follow(
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let target = player.translation.with_z(camera.translation.z);
    camera.translation.smooth_nudge(&target, 5.0, time.delta_secs()); // higher = snappier
}
```

> [!WARNING]
> Common mistakes: writing a child's `Transform` as if it were world space; two `&mut Transform` queries in one system without `Without<...>` ([[Advanced queries]]); forgetting `z` in 2D; moving a physics body by hand instead of through the engine ([[Physics]]).

## Next

[[Sprites]], then [[Cameras]].
