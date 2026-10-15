---
icon: vibrate
tags: [gameplay]
---
# Game feel

Two games with the same rules can feel completely different. The difference is **feedback**: how loudly the world answers when you act. None of these tricks change the rules; each is one small system. #gameplay

## A hit with no feedback

Click the square. You hear the hit (your own sound, if you made one with the speaker button), and that's all. It feels like nothing happened.

```rust example=feel
use bevy::prelude::*;

#[derive(Component)]
struct Player;

/// 1.0 right after a hit, falling back to 0.
#[derive(Component, Default)]
struct Squash(f32);

/// Blinking while the timer runs.
#[derive(Component)]
struct Blink(Timer);

#[derive(Resource)]
struct Sounds {
    hit: Handle<AudioSource>,
}

fn on_hit(
    click: On<PointerClick>,
    mut player: Query<&mut Squash, With<Player>>,
    sounds: Res<Sounds>,
    mut commands: Commands,
) {
    let Ok(mut squash) = player.get_mut(click.entity) else {
        return;
    };
    squash.0 = 1.0;
    commands
        .entity(click.entity)
        .insert(Blink(Timer::from_seconds(0.4, TimerMode::Once)));
    commands.spawn((AudioPlayer::new(sounds.hit.clone()), PlaybackSettings::DESPAWN));
}
```

Now add feedback, one block at a time. Each preview runs the hit plus that block.

## Squash and stretch

Wide and flat on impact, then back to square. An easing curve makes the recovery snappy instead of linear:

```rust example=feel
fn squash(time: Res<Time>, mut query: Query<(&mut Squash, &mut Transform)>) {
    for (mut squash, mut transform) in &mut query {
        squash.0 = (squash.0 - time.delta_secs() * 4.0).max(0.0);
        let s = EaseFunction::QuadraticOut.sample_clamped(squash.0);
        transform.scale = Vec3::new(1.0 + 0.4 * s, 1.0 - 0.35 * s, 1.0);
    }
}
```

## Blink

The classic "I got hurt" signal, and a hint that you're briefly invincible:

```rust example=feel
fn blink(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Blink, &mut Visibility)>,
    mut commands: Commands,
) {
    for (entity, mut blink, mut visibility) in &mut query {
        blink.0.tick(time.delta());
        let off = blink.0.elapsed_secs() * 20.0 % 2.0 < 1.0;
        *visibility = if off && !blink.0.is_finished() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if blink.0.is_finished() {
            commands.entity(entity).remove::<Blink>();
        }
    }
}
```

## Sparks

A handful of tiny sprites that fly out, slow down and shrink away. No particle engine needed:

```rust example=feel
#[derive(Component)]
struct Spark {
    velocity: Vec2,
    life: Timer,
}

fn burst(click: On<PointerClick>, player: Query<&Transform, With<Player>>, mut commands: Commands) {
    let Ok(transform) = player.get(click.entity) else {
        return;
    };
    for i in 0..8 {
        let angle = i as f32 / 8.0 * std::f32::consts::TAU;
        commands.spawn((
            Spark {
                velocity: Vec2::from_angle(angle) * 120.0,
                life: Timer::from_seconds(0.35, TimerMode::Once),
            },
            Sprite::from_color(Color::srgb(0.98, 0.75, 0.3), Vec2::splat(5.0)),
            Transform::from_translation(transform.translation.with_z(2.0)),
        ));
    }
}

fn sparks(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Spark, &mut Transform)>,
    mut commands: Commands,
) {
    for (entity, mut spark, mut transform) in &mut query {
        spark.life.tick(time.delta());
        transform.translation += (spark.velocity * time.delta_secs()).extend(0.0);
        spark.velocity *= (1.0 - 6.0 * time.delta_secs()).max(0.0); // fast out, then hang
        transform.scale = Vec3::splat(1.0 - spark.life.fraction());
        if spark.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

For thousands of particles, use [[bevy_hanabi]] on the GPU.

## Screen shake

Random offsets look jittery and stack badly. Instead keep a **trauma** value from 0 to 1 that decays, and shake by `trauma²`: small hits barely move the camera, big ones really kick.

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Trauma(f32);

/// Where the camera really is; the shake is drawn on top.
#[derive(Component)]
struct Shake {
    origin: Vec3,
}

fn on_big_hit(mut trauma: ResMut<Trauma>) {
    trauma.0 = (trauma.0 + 0.4).min(1.0);
}

fn shake_camera(
    time: Res<Time>,
    mut trauma: ResMut<Trauma>,
    mut camera: Single<(&mut Transform, &Shake), With<Camera2d>>,
) {
    let t = time.elapsed_secs() * 40.0;
    let power = trauma.0 * trauma.0;
    // Two sines at unrelated speeds stand in for smooth noise.
    let offset = Vec2::new(t.sin() + (t * 2.3).sin(), (t * 1.7).cos() + (t * 3.1).sin()) * 0.5;
    let (transform, shake) = &mut *camera;
    transform.translation = shake.origin + (offset * power * 12.0).extend(0.0);
    transform.rotation = Quat::from_rotation_z(offset.x * power * 0.03);
    trauma.0 = (trauma.0 - time.delta_secs() * 1.2).max(0.0);
}
```

A camera that follows the player ([[Cameras]]) moves `Shake::origin`, so the two never fight.

## Hit stop

On a big hit, **freeze the game for a few frames**. Players read it as weight. Slow `Time<Virtual>` (everything that uses the normal clock) and bring it back on a real-time timer:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct HitStop(Timer);

fn start_hit_stop(mut commands: Commands, mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.set_relative_speed(0.05);
    commands.insert_resource(HitStop(Timer::from_seconds(0.08, TimerMode::Once)));
}

fn end_hit_stop(
    real: Res<Time<Real>>,
    stop: Option<ResMut<HitStop>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut commands: Commands,
) {
    let Some(mut stop) = stop else { return };
    if stop.0.tick(real.delta()).is_finished() {
        virtual_time.set_relative_speed(1.0);
        commands.remove_resource::<HitStop>();
    }
}
```

Eighty milliseconds is plenty; longer feels like lag.

## Easing

`EaseFunction` turns a linear 0..1 progress into a shaped one. Pick by feel:

| Curve | Feels | Use for |
|:--|:--|:--|
| `QuadraticOut` | fast start, soft landing | things arriving |
| `QuadraticIn` | slow start, fast end | things leaving |
| `BackOut` | overshoots, settles | pop-ups, pickups |
| `ElasticOut` | wobbles | rewards, jelly |
| `BounceOut` | bounces | things dropping |

```rust
use bevy::prelude::*;

/// Grows past full size, then settles.
#[derive(Component)]
struct Pop(Timer);

fn pop(time: Res<Time>, mut query: Query<(&mut Pop, &mut Transform)>) {
    for (mut pop, mut transform) in &mut query {
        pop.0.tick(time.delta());
        transform.scale = Vec3::splat(EaseFunction::BackOut.sample_clamped(pop.0.fraction()));
    }
}
```

Many tweens at once? [[bevy_tweening]] wraps this in components.

## The rest of the toolbox

- [ ] **Sound** on every action, with a little random pitch ([[Audio]], [[Randomness]]).
- [ ] **Anticipation**: a short wind-up before a big attack gives players time to react.
- [ ] **Coyote time**: allow a jump for ~0.1 s after leaving a ledge. Platformers feel fair.
- [ ] **Knockback**: push the hit thing away from the hit.

> [!TIP]
> Add them in this order: sound, squash, sparks, shake, hit stop. Sound and a shake alone make a punch feel twice as strong.

## Next

[[Audio]], [[Cameras]], or a full visual pass: [[Make it look good]].
