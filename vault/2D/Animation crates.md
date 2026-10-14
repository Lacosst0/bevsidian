---
icon: package
tags: [2d, 3d, ui]
bevy: 0.19
crates: bevy_tweening=0.16; bevy_spritesheet_animation=7.0.1
---
# Animation crates

Two crates that animate for you: tweens for any value, and clips for sprite sheets. For the built-in way see [[Sprite animation]].

## bevy_tweening

**Tweens**: animate any value (position, scale, colour, a UI size, your own component) from A to B over time with easing, loops and sequences. The easiest way to add juice. #2d #3d #ui

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): the latest release `0.16` targets **Bevy 0.19**; there is no 0.20 release yet. The code below compiles against 0.19 + `bevy_tweening 0.16`. Watch the [repo](https://github.com/djeedai/bevy_tweening) and the table in [[2D]].

```toml
bevy_tweening = "0.16"
```

### Move something, smoothly

```rust
use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{lens::*, *};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TweeningPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        TransformPositionLens {
            start: Vec3::new(-150.0, 0.0, 0.0),
            end: Vec3::new(150.0, 0.0, 0.0),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);   // there and back

    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.4, 0.3), Vec2::splat(50.0)),
        TweenAnim::new(tween),      // the component that plays it
    ));
}
```

A `Tween` is *easing function + duration + lens*. The **lens** knows how to write a value onto a component for a ratio from 0 to 1. The `TweenAnim` component plays it on the same entity (add an `AnimTarget` to animate another entity, a resource or an asset).

### Shortcuts and sequences

```rust
use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{lens::*, *};

fn pop(mut commands: Commands) {
    // Built-in helper: move_to.
    commands
        .spawn(Transform::default())
        .move_to(Vec3::new(100.0, 50.0, 0.0), Duration::from_millis(400), EaseFunction::BackOut);

    // Chain two tweens: grow, then shrink.
    let grow = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(150),
        TransformScaleLens { start: Vec3::ONE, end: Vec3::splat(1.3) },
    );
    let shrink = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(150),
        TransformScaleLens { start: Vec3::splat(1.3), end: Vec3::ONE },
    );
    commands.spawn((Transform::default(), TweenAnim::new(grow.then(shrink))));
}
```

Use sequences for button presses, pickups and hit flashes.

### Your own lens

Tween any component by writing a lens:

```rust
use bevy::prelude::*;
use bevy_tweening::{lens::*, *};

#[derive(Component)]
struct Glow(f32);

struct GlowLens {
    start: f32,
    end: f32,
}

impl Lens<Glow> for GlowLens {
    fn lerp(&mut self, mut target: Mut<Glow>, ratio: f32) {
        target.0 = self.start + (self.end - self.start) * ratio;
    }
}
```

### When to use it

- Short, scripted motion: UI transitions, pop-ins, camera nudges, colour flashes.
- Not for physics or anything that reacts to the player each frame: write a system ([[Systems]]).
- For easing maths without the crate: `EaseFunction::sample` ([[Colors and math]]).

### Links

[crates.io](https://crates.io/crates/bevy_tweening) · [docs.rs](https://docs.rs/bevy_tweening) · [repo](https://github.com/djeedai/bevy_tweening)

## bevy_spritesheet_animation

Sprite-sheet animation **without writing the timer system** ([[Sprite animation]]). Define clips from rows, columns or frame lists; set speed, easing and repeats; combine clips and react to events. #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `7.0.1` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_spritesheet_animation = "7.0"
```

### A looping animation from one row

```rust
use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(SpritesheetAnimationPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut animations: ResMut<Assets<Animation>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let image = assets.load("character.png");
    let spritesheet = Spritesheet::new(&image, 8, 8);          // 8 columns, 8 rows

    let run = spritesheet
        .create_animation()
        .add_row(3)                                            // frames from row 3
        .set_duration(AnimationDuration::PerFrame(100))        // ms per frame
        .build();

    commands.spawn((
        spritesheet.with_size_hint(768, 768).sprite(&mut layouts),
        SpritesheetAnimation::new(animations.add(run)),
    ));
}
```

### Several clips, switch at runtime

Build one `Animation` per clip (idle, run, shoot) and swap the handle in `SpritesheetAnimation` when the character's state changes ([[Change detection]]). Easing, repetition, clip chaining and **animation events** (footstep at frame 3) are part of the builder.

### Versus doing it yourself

| | Hand-written ([[Sprite animation]]) | This crate |
|:--|:--|:--|
| Code | ~30 lines you own | a few builder calls |
| Features | what you add | easing, events, composition, many clips |
| Dependency | none | yes |

For one animated character, the hand-written system is perfectly fine.

### Links

[crates.io](https://crates.io/crates/bevy_spritesheet_animation) · [docs.rs](https://docs.rs/bevy_spritesheet_animation) · [repo](https://github.com/merwaaan/bevy_spritesheet_animation)
