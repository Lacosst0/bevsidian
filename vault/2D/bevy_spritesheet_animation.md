---
lib: true
parent: Animation crates
icon: package
tags: [2d]
bevy: 0.19
crates: bevy_spritesheet_animation=7.0.1
---
# bevy_spritesheet_animation

Sprite-sheet animation **without writing the timer system** ([[Sprite animation]]). Define clips from rows, columns or frame lists; set speed, easing and repeats; combine clips and react to events. #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `7.0.1` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_spritesheet_animation = "7.0"
```

## A looping animation from one row

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

## Several clips, switch at runtime

Build one `Animation` per clip (idle, run, shoot) and swap the handle in `SpritesheetAnimation` when the character's state changes ([[Change detection]]). Easing, repetition, clip chaining and **animation events** (footstep at frame 3) are part of the builder.

## Versus doing it yourself

| | Hand-written ([[Sprite animation]]) | This crate |
|:--|:--|:--|
| Code | ~30 lines you own | a few builder calls |
| Features | what you add | easing, events, composition, many clips |
| Dependency | none | yes |

For one animated character, the hand-written system is perfectly fine.

## Links

[crates.io](https://crates.io/crates/bevy_spritesheet_animation) · [docs.rs](https://docs.rs/bevy_spritesheet_animation) · [repo](https://github.com/merwaaan/bevy_spritesheet_animation)

## Next

[[2D]].
