---
icon: package
tags: [gameplay]
crates: bevy_rand={ version = "0.16", features = ["wyrand"] }; bevy_prng={ version = "0.16", features = ["wyrand"] }; rand_core="0.10"
---
# bevy_rand

**Random numbers the ECS way.** Bevy has no random number generator of its own; this crate wraps the `rand` family so generators are components and resources, can be **seeded** (reproducible runs, replays, networked simulations) and **forked** per entity. #gameplay

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_rand 0.16` supports **Bevy 0.20**.

```toml
bevy_rand = { version = "0.16", features = ["wyrand"] }   # a fast, portable generator
bevy_prng = { version = "0.16", features = ["wyrand"] }
rand_core = "0.10"
```

On the web, also enable the `wasm_js` feature ([[Web build]]).

## Set up and use

Register a generator type once, then ask for the global one:

```rust
use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use rand_core::Rng;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntropyPlugin::<WyRand>::default()))
        .add_systems(Startup, roll)
        .run();
}

fn roll(mut rng: Single<&mut WyRand, With<GlobalRng>>) {
    let die = rng.next_u32() % 6 + 1;
    info!("you rolled a {die}");
}
```

`GlobalRng` is one shared generator: simple, but systems that use it cannot run in parallel.

## One generator per entity

Give each enemy its own seeded generator by **forking** the global one. Now the enemies run in parallel, and the whole game stays reproducible from one seed:

```rust
use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use rand_core::Rng;

#[derive(Component)]
struct Enemy;

fn spawn_enemies(mut commands: Commands, mut global: Single<&mut WyRand, With<GlobalRng>>) {
    for _ in 0..10 {
        commands.spawn((Enemy, global.fork_seed()));
    }
}

fn wander(mut enemies: Query<(&mut Transform, &mut WyRand), With<Enemy>>) {
    for (mut transform, mut rng) in &mut enemies {
        let step = (rng.next_u32() % 3) as f32 - 1.0;
        transform.translation.x += step;
    }
}
```

## Fixed seed for tests and replays

Insert the plugin with a seed (`EntropyPlugin::<WyRand>::with_seed(..)`) and every run gives the same numbers. The same trick is how networked games keep the server and clients in step ([[Networking]]).

## Which algorithm?

`wyrand` for almost everything. `chacha20` if you need stronger statistical quality. **Not for security.**

## Links

[crates.io](https://crates.io/crates/bevy_rand) · [docs.rs](https://docs.rs/bevy_rand) · [repo](https://github.com/Bluefinger/bevy_rand)

## Next

[[Gameplay]].
