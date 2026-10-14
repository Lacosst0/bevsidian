---
parent: Cameras
icon: sparkles
tags: [rendering]
---
# Post-processing

Effects applied to the finished image: glow, colour grading, edge smoothing. Each one is a component you add to the **camera**. #rendering

## Bloom: glow

```rust
use bevy::{post_process::bloom::Bloom, prelude::*};

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Bloom::NATURAL,       // also Bloom::OLD_SCHOOL, Bloom::ANAMORPHIC
    ));
    commands.spawn(Sprite::from_color(
        Color::srgb(6.0, 9.0, 9.0),   // above 1.0 = brighter than white = glows
        Vec2::splat(60.0),
    ));
}
```

Bloom only lights up what is brighter than 1.0, so give your glowing things HDR colours. In 3D, also make sure the camera has `Hdr` (see Bevy's `bloom_3d` example).

## Tone mapping

3D lighting produces values far above 1.0. A **tone mapper** squeezes them into what a screen can show:

```rust
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

fn camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Tonemapping::TonyMcMapface));
}
```

`TonyMcMapface` is the default and good for most games. `AcesFitted`, `AgX`, `Reinhard` and `None` are there to match a look.

## Anti-aliasing

Smooths jagged edges:

| Option | Cost | Notes |
|:--|:--|:--|
| `Msaa::Sample4` (default in 3D) | medium | on the camera; not with deferred rendering |
| `Smaa` | low | good all-rounder, works with HDR |
| `TemporalAntiAlias` | low | smoothest, needs motion; add `TemporalJitter` |
| `Fxaa` | lowest | blurry |

```rust
use bevy::{anti_alias::smaa::Smaa, prelude::*};

fn camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Msaa::Off, Smaa::default()));
}
```

## Colour grading, exposure, fog

- `ColorGrading` on the camera: exposure, contrast, saturation, temperature.
- `DistanceFog` on the camera: tint far-away objects. See [[Lighting]].
- `AutoExposure`: adapts to dark and bright scenes like an eye.
- `ScreenSpaceAmbientOcclusion`, `DepthOfField`, `MotionBlur`: richer 3D looks, each its own component.

All of them follow the same rule: *add the component to the camera; remove it to turn it off*.

## Cost

Effects cost GPU time, and on the web some are unavailable. Start with bloom and tone mapping; add the rest when you can see why.

## Make it look good

The path [[Make it look good]] ties these together.

## Next

[[Custom materials]].
