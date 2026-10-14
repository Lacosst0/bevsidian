---
icon: palette
tags: [rendering]
---
# Colors and math

The small types you use everywhere: [`Color`], `Vec2`, `Vec3`, `Quat`. #rendering

## Colors

`Color` is an enum of colour spaces. Use the constructor that matches how you think:

```rust
use bevy::prelude::*;

fn colors() {
    let red = Color::srgb(1.0, 0.0, 0.0);              // 0.0 to 1.0
    let teal = Color::srgb_u8(0, 128, 128);            // 0 to 255
    let from_hex = Color::from(Srgba::hex("#ff8800").unwrap());
    let sky = Color::hsl(210.0, 0.8, 0.6);             // hue in degrees
    let faded = Color::srgba(1.0, 1.0, 1.0, 0.5);      // with alpha
    let white = Color::WHITE;                          // constants (also css::TOMATO etc.)
    let tomato = bevy::color::palettes::css::TOMATO;   // named colours live in palettes
}
```

### Mix colours

HSL is the friendly way to *choose* a colour: hue is the position on the colour wheel in degrees (0 red, 120 green, 240 blue), saturation the strength, lightness from black to white. `mix` blends two colours. Try it:

```rust preview=colour-lab
use bevy::prelude::*;

fn lab(hue: f32, lightness: f32, amount: f32) -> (Color, Color) {
    let base = Color::hsl(hue, 0.8, lightness);
    let mixed = base.mix(&Color::WHITE, amount);          // 0 = base, 1 = white
    (base, mixed)
}

fn hex(color: Color) -> String {
    Srgba::from(color).to_hex()                           // "#3377EE"
}
```

### Change a colour

```rust
use bevy::prelude::*;

fn tweak() {
    let base = Color::srgb(0.2, 0.5, 0.9);
    let lighter = base.lighter(0.2);
    let see_through = base.with_alpha(0.3);
    let mixed = base.mix(&Color::WHITE, 0.5);           // halfway to white
}
```

### Brighter than white

A colour's channels can exceed 1.0. In a camera with `Bloom` ([[Post-processing]]) that glows. `Color::srgb(6.0, 9.0, 9.0)` is a neon light.

## Vectors

```rust
use bevy::prelude::*;

fn vectors() {
    let a = Vec2::new(3.0, 4.0);
    let b = Vec2::X;                       // (1, 0); also Y, NEG_X, ZERO, ONE
    let length = a.length();               // 5.0
    let direction = a.normalize();         // length 1
    let towards = (b - a).normalize_or_zero();
    let distance = a.distance(b);
    let dot = a.dot(b);
    let mixed = a.lerp(b, 0.25);           // 25% of the way from a to b
    let clamped = a.clamp_length_max(2.0);
    let v3 = a.extend(0.0);                // Vec2 -> Vec3
    let v2 = v3.truncate();                // Vec3 -> Vec2
}
```

A `Vec2` for 2D positions, `Vec3` for 3D (and 2D with `z` for draw order).

## Rotations

Rotation is a quaternion, `Quat`. Build one from something you understand:

```rust
use bevy::prelude::*;

fn rotations() {
    let turn = Quat::from_rotation_z(90.0_f32.to_radians());        // around z
    let tilt = Quat::from_euler(EulerRot::XYZ, 0.3, 0.0, 0.0);
    let aim = Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y).rotation;
    let both = turn * tilt;                                         // combine
    let rotated = turn * Vec3::X;                                   // rotate a vector
    let halfway = turn.slerp(tilt, 0.5);                            // smooth blend
}
```

Angles are **radians**. `x.to_radians()` and `x.to_degrees()` convert.

## Shapes and rectangles

Primitive shapes (`Circle`, `Rectangle`, `Cuboid`, `Sphere`...) describe geometry. Turn them into meshes with `meshes.add(...)` ([[2D shapes and meshes]], [[Meshes and materials]]). `Rect` and `Aabb2d` handle overlap tests:

```rust
use bevy::prelude::*;
use bevy::shape::{Aabb2d, IntersectsVolume};

fn overlap() -> bool {
    let a = Aabb2d::new(Vec2::new(0.0, 0.0), Vec2::splat(10.0));   // centre, half size
    let b = Aabb2d::new(Vec2::new(15.0, 0.0), Vec2::splat(10.0));
    a.intersects(&b)
}
```

## Easing and curves

`EaseFunction` turns a 0 to 1 progress into a shaped 0 to 1. Pick a curve and watch the ball: the same time, a different feel.

```rust preview=easing
use bevy::prelude::*;

fn ease(t: f32) -> f32 {
    EaseFunction::CubicInOut.sample_clamped(t)      // try QuadraticOut, BackOut, ElasticOut, BounceOut
}

fn blend(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    from.lerp(to, ease(t))
}
```

## Random numbers

Bevy has no built-in RNG type. Add `rand` (or [[bevy_rand]]); [[Randomness]] shows how:

```toml
rand = "0.10"
```

## Next

[[Post-processing]].
