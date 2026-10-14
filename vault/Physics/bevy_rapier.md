---
lib: true
icon: package
tags: [physics]
crates: bevy_rapier2d=0.37
---
# bevy_rapier

The official Bevy integration of **Rapier**, a mature, fast physics engine. Rigid bodies, colliders, joints, character controllers, ray casts, sensors. 2D (`bevy_rapier2d`) and 3D (`bevy_rapier3d`). #physics

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): release `0.37` supports **Bevy 0.20**. The plugin lives in the main Rapier repository now.

```toml
bevy_rapier2d = "0.37"     # or bevy_rapier3d = "0.37"
```

## The model

You give an entity three kinds of things:

| Component | Meaning |
|:--|:--|
| `RigidBody::Dynamic` / `Fixed` / `KinematicPositionBased` | how the body moves: simulated, immovable, or moved by you |
| `Collider::ball(r)`, `cuboid(hx, hy)`, `capsule_y(..)`, ... | its shape |
| `Velocity`, `Restitution`, `Friction`, `GravityScale`, `Sensor` | how it behaves |

Rapier then writes the result into the entity's `Transform` every frame. **Do not move simulated bodies by editing `Transform`**; set velocity or apply forces.

## A falling ball

```rust
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0), // 100 px = 1 m
            RapierDebugRenderPlugin::default(),                         // draws the colliders
        ))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, report_hits)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // The floor: no RigidBody means fixed.
    commands.spawn((
        Transform::from_xyz(0.0, -200.0, 0.0),
        Collider::cuboid(300.0, 20.0),
    ));

    // A bouncy ball.
    commands.spawn((
        Transform::from_xyz(0.0, 150.0, 0.0),
        RigidBody::Dynamic,
        Collider::ball(25.0),
        Restitution::coefficient(0.8),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

fn report_hits(mut collisions: MessageReader<CollisionEvent>) {
    for event in collisions.read() {
        if let CollisionEvent::Started(a, b, _) = event {
            info!("{a} hit {b}");
        }
    }
}
```

Collision events are [[Messages|messages]]: read them with a `MessageReader`.

## Moving a character

Apply velocity, let the engine handle walls and floors:

```rust
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)] struct Player;

fn walk(keys: Res<ButtonInput<KeyCode>>, mut player: Single<&mut Velocity, With<Player>>) {
    let x = keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8;
    player.linear.x = x as f32 * 200.0;
}
```

For slopes, steps and "walk, don't slide", use the `KinematicCharacterController` component (see the `character_controller` examples).

## Sensors and ray casts

- Add `Sensor` to a collider to detect overlaps without pushing (pickups, trigger zones).
- Ask the `ReadRapierContext` system parameter for ray casts, shape casts and intersection tests.

## Fixed time step

Rapier steps in `PostUpdate` by default, tied to frame time; configure `TimestepMode::Fixed` to be deterministic, and interpolate visuals ([[bevy_transform_interpolation]]).

## Compared to Avian

| | bevy_rapier | [[Avian]] |
|:--|:--|:--|
| Engine | Rapier (separate, mature, very feature complete) | written for Bevy's ECS |
| Style | components, plugin config | components, more "Bevy-native" |
| Bevy 0.20 | **released** | release pending |

## Links

[crates.io](https://crates.io/crates/bevy_rapier2d) · [docs.rs](https://docs.rs/bevy_rapier2d) · [Rapier user guide](https://rapier.rs/docs/) · [repo](https://github.com/dimforge/rapier/tree/master/bindings/bevy_rapier)

## Next

[[Physics]].
