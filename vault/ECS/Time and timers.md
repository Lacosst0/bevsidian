---
icon: clock
tags: [ecs, intermediate]
---
# Time and timers

Games run at different frame rates on different machines. Always move things by **time**, not by frame. #ecs #intermediate

## The Time resource

```rust
use bevy::prelude::*;

#[derive(Component)] struct Speed(f32);

fn drift(time: Res<Time>, mut query: Query<(&mut Transform, &Speed)>) {
    for (mut transform, speed) in &mut query {
        // Units per second × seconds since the last frame.
        transform.translation.x += speed.0 * time.delta_secs();
    }
}
```

| Method | Meaning |
|:--|:--|
| `delta_secs()` | seconds since the previous frame |
| `elapsed_secs()` | seconds since the app started |
| `delta()` / `elapsed()` | the same as a `Duration` |

Multiply every speed by `delta_secs()`. That is the whole rule.

## Timer

A [`Timer`] counts up to a duration. You feed it the frame's time:

```rust example=timers fps=20
use bevy::prelude::*;

#[derive(Component)]
struct Clock(Timer);

#[derive(Component)]
struct Ticks(u32);

fn spawn_clock(mut commands: Commands) {
    commands.spawn((
        Name::new("Clock"),
        Clock(Timer::from_seconds(0.5, TimerMode::Repeating)),
        Ticks(0),
    ));
}

fn tick_clock(time: Res<Time>, mut clocks: Query<(&mut Clock, &mut Ticks)>) {
    for (mut clock, mut ticks) in &mut clocks {
        // `tick` adds this frame's time; `just_finished` is true on the frame it wraps.
        if clock.0.tick(time.delta()).just_finished() {
            ticks.0 += 1;
        }
    }
}
```

Play it. Every 30 frames (half a second at the preview's 60 simulated frames per second) `Ticks` goes up.

| Mode | After the duration |
|:--|:--|
| `TimerMode::Once` | stops; `finished()` stays true |
| `TimerMode::Repeating` | starts over; `just_finished()` is true for one frame |

Handy methods: `reset()`, `pause()`, `fraction()` (0 to 1, perfect for progress bars), `remaining_secs()`.

### Cooldowns

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Gun { cooldown: Timer }

fn shoot(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut guns: Query<&mut Gun>) {
    for mut gun in &mut guns {
        gun.cooldown.tick(time.delta());
        if keys.pressed(KeyCode::Space) && gun.cooldown.is_finished() {
            gun.cooldown.reset();
            // fire!
        }
    }
}
```

## Fixed timestep

`Update` runs once per frame, so its delta varies. Physics and game rules want **the same step every time**. Put those in `FixedUpdate`, which runs zero or more times per frame at 64 Hz by default:

```rust
use bevy::prelude::*;

fn simulate(time: Res<Time>) {
    // Inside FixedUpdate, `Time` is the fixed clock: delta is always 1/64 s.
    let _dt = time.delta_secs();
}

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(30.0))   // change the rate
        .add_systems(FixedUpdate, simulate)
        .run();
}
```

- Read input in `Update`, store what the player wants, apply it in `FixedUpdate`.
- Drawing a fixed-rate world smoothly needs interpolation; see the transform interpolation crates in [[Physics]].

## Virtual time: slow motion and pause

`Time<Virtual>` scales the clock that `Res<Time>` shows in `Update`:

```rust
use bevy::prelude::*;

fn slow_mo(keys: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    if keys.just_pressed(KeyCode::KeyT) {
        time.set_relative_speed(0.25);   // quarter speed
    }
    if keys.just_pressed(KeyCode::KeyP) {
        if time.is_paused() { time.unpause() } else { time.pause() }
    }
}
```

`Time<Real>` ignores the scale: use it for menus and UI animations that must keep running when the game is paused.

## Delays without a timer component

```rust
use bevy::prelude::*;

#[derive(Component)] struct Bomb;

fn setup(mut commands: Commands) {
    let bomb = commands.spawn(Bomb).id();
    // These commands run two seconds from now.
    commands.delayed().secs(2.0).entity(bomb).despawn();
}
```

## Next

[[Error handling]].
