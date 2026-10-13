---
icon: package
tags: [lib, 2d, 3d]
bevy: 0.19
crates: bevy_hanabi=0.19
---
# bevy_hanabi

**GPU particles**: fire, smoke, sparks, magic, rain; hundreds of thousands of particles simulated on the graphics card. You describe an *effect* with modifiers, then spawn it as an entity. #lib #2d #3d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.19` targets **Bevy 0.19**; the repository's `main` is at `0.20.0-dev`. The code compiles against 0.19.

```toml
bevy_hanabi = "0.19"
# 2D only:  bevy_hanabi = { version = "0.19", default-features = false, features = ["2d"] }
```

Needs a GPU with compute shaders: **no WebGL2**; works on WebGPU ([[Web build]]).

## An effect

An effect is an asset: *when particles are born* (`init`), *how they change* (`update`), and *how they look* (`render`).

```rust
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HanabiPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, 3.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y)));

    // Colour over lifetime: red → transparent.
    let mut gradient = bevy_hanabi::Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 0.3, 0.0, 1.0));
    gradient.add_key(1.0, Vec4::splat(0.0));

    let mut module = Module::default();

    // Born on the surface of a small sphere...
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.5),
        dimension: ShapeDimension::Surface,
    };
    // ...flying outwards...
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(4.0),
    };
    // ...living two seconds (always set a lifetime or nothing shows).
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(2.0));
    // Gravity-like pull each frame.
    let update_accel = AccelModifier::new(module.lit(Vec3::new(0.0, -3.0, 0.0)));

    let effect = EffectAsset::new(1024, SpawnerSettings::rate(50.0.into()), module)
        .with_name("sparks")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .update(update_accel)
        .render(ColorOverLifetimeModifier { gradient, ..default() });

    commands.spawn((ParticleEffect::new(effects.add(effect)), Transform::from_xyz(0.0, 1.0, 0.0)));
}
```

Move the entity and the emitter moves ([[Transforms]]); despawn it to stop.

## Ideas

- Bursts: `SpawnerSettings::once(100.0.into())` for explosions, triggered by an [[Events and observers|observer]].
- Add `bevy_hanabi` effects to hit events: spawn an effect entity, let it despawn after its lifetime.
- Combine with emissive HDR colours and [[Post-processing|bloom]] for glow.

## Not for you if...

You only need a few dozen sprites flying around: spawn ordinary [[Sprites]] and move them with a system ([[Physics]] shows how).

## Links

[crates.io](https://crates.io/crates/bevy_hanabi) · [docs.rs](https://docs.rs/bevy_hanabi) · [repo](https://github.com/djeedai/bevy_hanabi) · examples in the repo (`firework`, `gradient`, `portal`...)

## Next

[[3D]], [[2D]], [[Libraries]].
