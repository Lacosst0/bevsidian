---
icon: vibrate
tags: [gameplay]
---
# Game feel

Two games can have identical rules and feel completely different. The difference is **feedback**: how much the world answers when you act. These are the cheap tricks that carry most of it. Each is a small system. #gameplay

> [!TIP]
> Add them in this order: **sound, screen shake, hit stop, easing**. A punch with a sound and a shake already feels twice as strong, and nothing here changes the rules of the game.

## Screen shake with trauma

Naive shake adds a random offset. It looks jittery and stacks badly. The standard trick is a **trauma** value from 0 to 1 that decays, and a shake strength of `trauma²`. Hits add trauma; the camera shakes by a smooth noise scaled by it.

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Trauma(f32);

#[derive(Component)]
struct Shake { origin: Vec3 }

/// Anything that hurts adds trauma (0..1).
fn on_hit(mut trauma: ResMut<Trauma>) {
    trauma.0 = (trauma.0 + 0.4).min(1.0);
}

fn shake_camera(
    time: Res<Time>,
    mut trauma: ResMut<Trauma>,
    mut camera: Single<(&mut Transform, &Shake), With<Camera2d>>,
) {
    let t = time.elapsed_secs() * 40.0;
    let power = trauma.0 * trauma.0;
    // Two sine waves at unrelated speeds stand in for smooth noise.
    let offset = Vec2::new((t * 1.0).sin() + (t * 2.3).sin(), (t * 1.7).cos() + (t * 3.1).sin()) * 0.5;
    let (transform, shake) = &mut *camera;
    transform.translation = shake.origin + (offset * power * 12.0).extend(0.0);
    transform.rotation = Quat::from_rotation_z(offset.x * power * 0.03);
    trauma.0 = (trauma.0 - time.delta_secs() * 1.2).max(0.0);
}
```

Keep the camera's real position in `Shake::origin` and write the shaken one on top, so a camera that follows the player ([[Cameras]]) does not fight the shake. For real noise use the function from [[Procedural generation]].

## Hit stop

On a strong hit, **freeze the game for a few frames**. Players read it as weight. `Time<Virtual>` slows everything that uses the normal clock ([[Time and timers]]); a real-time timer brings it back:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct HitStop(Timer);

fn start_hit_stop(mut commands: Commands, mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.set_relative_speed(0.05);                 // almost frozen
    commands.insert_resource(HitStop(Timer::from_seconds(0.08, TimerMode::Once)));
}

fn end_hit_stop(
    real: Res<Time<Real>>,
    stop: Option<ResMut<HitStop>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut commands: Commands,
) {
    let Some(mut stop) = stop else { return };
    if stop.0.tick(real.delta()).is_finished() {           // real time, not slowed
        virtual_time.set_relative_speed(1.0);
        commands.remove_resource::<HitStop>();
    }
}
```

Eighty milliseconds is enough. Longer feels like lag.

## Easing

Linear motion looks mechanical. **Easing** curves speed up and slow down. Bevy has them built in as `EaseFunction`; sample one to turn a 0..1 progress into a shaped 0..1:

```rust
use bevy::prelude::*;

/// A button that "pops" when it appears: grows past full size, then settles.
#[derive(Component)]
struct Pop(Timer);

fn pop(time: Res<Time>, mut buttons: Query<(&mut Pop, &mut Transform)>) {
    for (mut pop, mut transform) in &mut buttons {
        pop.0.tick(time.delta());
        let eased = EaseFunction::BackOut.sample_clamped(pop.0.fraction());
        transform.scale = Vec3::splat(eased);
    }
}
```

| Curve | Feels | Use for |
|:--|:--|:--|
| `QuadraticOut` | fast start, soft landing | things arriving, UI sliding in |
| `QuadraticIn` | slow start, fast end | things leaving |
| `BackOut` | overshoots, settles | pop-ups, pickups |
| `ElasticOut` | wobbles | rewards, jelly |
| `BounceOut` | bounces | things dropping |

For many animations at once, [[bevy_tweening]] wraps all this in components.

## Squash and stretch

Scale a sprite a little on jump and landing. Tall and thin going up, wide and flat on impact, back to 1 over ~0.15 s using an easing curve:

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
struct Squash(f32);   // 1.0 right after landing, falls to 0

fn squash(time: Res<Time>, mut query: Query<(&mut Squash, &mut Transform)>) {
    for (mut squash, mut transform) in &mut query {
        squash.0 = (squash.0 - time.delta_secs() * 6.0).max(0.0);
        transform.scale = Vec3::new(1.0 + 0.3 * squash.0, 1.0 - 0.3 * squash.0, 1.0);
    }
}
```

## Everything else

- **Particles** on impact: [[bevy_hanabi]] for the GPU, or a handful of small sprites with a lifetime ([[Time and timers]]).
- **Sound** on every action, with slight random pitch ([[Audio]], [[Randomness]]).
- **Flash** the sprite white for two frames when it is hit.
- **Anticipation**: a short wind-up before a big attack tells the player to react.
- **Coyote time**: let the player jump for ~0.1 s after leaving a ledge. It makes platformers feel fair.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Grounded { since_left: Timer }

fn can_jump(grounded: &Grounded, on_floor: bool) -> bool {
    on_floor || !grounded.since_left.is_finished()
}
```

## Next

[[Audio]], [[Cameras]], and a full pass: [[Make it look good]].
