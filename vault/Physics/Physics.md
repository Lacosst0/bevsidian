---
icon: atom
tags: [physics, intermediate]
---
# Physics

Bevy has **no physics engine built in**. For simple games you write a few systems yourself; for anything with stacking, rolling or joints you add a physics library. #physics #intermediate

## Do it yourself first

Many games need only gravity, a floor and rectangles that overlap. That is a few lines, and you control every detail. Balls with gravity and bounce; click the stage to drop another:

```rust example=bouncing
use bevy::prelude::*;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ball {
    radius: f32,
}

const GRAVITY: f32 = -400.0;
const FLOOR: f32 = -80.0;
const HALF_WIDTH: f32 = 160.0;

fn ball(radius: f32, at: Vec2, velocity: Vec2) -> impl Bundle {
    (
        Ball { radius },
        Velocity(velocity),
        Sprite::from_color(Color::srgb(0.35, 0.75, 0.9), Vec2::splat(radius * 2.0)),
        Transform::from_translation(at.extend(1.0)),
    )
}

fn spawn_balls(mut commands: Commands) {
    commands.spawn(ball(10.0, Vec2::new(-60.0, 40.0), Vec2::new(30.0, 0.0)));
    commands.spawn(ball(14.0, Vec2::new(50.0, 60.0), Vec2::new(-20.0, 0.0)));
}

fn gravity(time: Res<Time>, mut balls: Query<(&mut Transform, &mut Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut balls {
        velocity.0.y += GRAVITY * dt;                       // acceleration changes velocity
        transform.translation += (velocity.0 * dt).extend(0.0);   // velocity changes position
    }
}

fn bounce(mut balls: Query<(&mut Transform, &mut Velocity, &Ball)>) {
    for (mut transform, mut velocity, ball) in &mut balls {
        // Floor: flip the speed and lose a little energy.
        if transform.translation.y - ball.radius < FLOOR && velocity.0.y < 0.0 {
            transform.translation.y = FLOOR + ball.radius;
            velocity.0.y = -velocity.0.y * 0.8;
        }
        // Walls: flip sideways.
        let wall = HALF_WIDTH - ball.radius;
        if transform.translation.x.abs() > wall {
            transform.translation.x = wall * transform.translation.x.signum();
            velocity.0.x = -velocity.0.x;
        }
    }
}
```

Step through it: `gravity` first adds to `velocity.y`, *then* moves. That order (accelerate, then move) is the heart of every physics engine. The factor `0.8` is "restitution": 1.0 would never lose energy.

```rust example=bouncing
#[derive(Component)]
struct Sky;

#[derive(Resource, Default)]
struct Dropped(usize);

fn drop_ball(click: On<PointerClick>, sky: Query<(), With<Sky>>, mut commands: Commands) {
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    commands.spawn(ball(8.0, at.truncate(), Vec2::ZERO));
}
```

## Box vs box

Axis-aligned boxes overlap when they overlap on **both** axes:

```rust
use bevy::prelude::*;

fn overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let gap = (a_pos - b_pos).abs() - (a_size + b_size) / 2.0;
    gap.x < 0.0 && gap.y < 0.0
}
```

For circles compare distance with the sum of the radii. Bevy's `Aabb2d`, `BoundingCircle` and ray casts ([[Colors and math]]) give you the same with more shapes.

## Keep it deterministic

Run gravity and collisions in `FixedUpdate` so results do not depend on frame rate ([[Time and timers]]):

```rust
use bevy::prelude::*;

fn gravity() {}
fn bounce() {}

fn main() {
    App::new()
        .add_systems(FixedUpdate, (gravity, bounce).chain())
        .run();
}
```

Because the world only changes at fixed steps, draw it smoothly with `bevy_transform_interpolation` ([[bevy_transform_interpolation]]).

## When to use a physics engine

You need one for: rigid bodies that stack and tumble, joints, many shapes colliding, continuous collision (fast bullets), character controllers on slopes, sensors and ray casts at scale.

| Engine | Style |
|:--|:--|
| [[bevy_rapier]] | wrapper of the mature Rapier engine; 2D and 3D; **has a Bevy 0.20 release** |
| [[Avian]] | ECS-native engine written for Bevy; 2D and 3D; follows Bevy closely |

Both give you components (`RigidBody`, `Collider`) and take over `Transform`.

## Next

[[Networking]] to put players together, or follow [[A 2D game]].
