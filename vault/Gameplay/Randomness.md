---
icon: dices
tags: [gameplay, math, beginner]
crates: rand="0.10"
---
# Randomness

Loot drops, enemy aim, particle sparks, a new level every run: games lean on random numbers. Rust does not ship a generator in `std`, and neither does Bevy, so you use the `rand` crate. #gameplay #math

```toml
rand = "0.10"
```

## Pick a number

Two ways in. The quick one is free functions that use a thread-local generator:

```rust
fn rolls() {
    let die: u32 = rand::random_range(1..=6);       // 1 to 6
    let angle: f32 = rand::random_range(0.0..std::f32::consts::TAU);
    let crit = rand::random_bool(0.1);               // true 10% of the time
}
```

The one you will use most in games is a generator **you own**, so you can seed it:

```rust
use rand::{Rng, RngExt, SeedableRng, rngs::StdRng};

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let a: u32 = rng.random_range(0..100);
    let b: u32 = rng.random_range(0..100);
    println!("{a} {b}");   // the same two numbers every run
}
```

`RngExt` is the trait that adds `random`, `random_range` and `random_bool` to every generator.

## Use it in a system

A generator is just data, so keep it in a resource. Seed it from a number you can log:

```rust
use bevy::prelude::*;
use rand::{RngExt, SeedableRng, rngs::StdRng};

#[derive(Resource)]
struct Dice(StdRng);

#[derive(Component, Default, Clone)]
struct Enemy;

fn spawn_wave(mut commands: Commands, mut dice: ResMut<Dice>) {
    for _ in 0..5 {
        let x = dice.0.random_range(-200.0..200.0);
        let y = dice.0.random_range(-100.0..100.0);
        commands.spawn((Enemy, Transform::from_xyz(x, y, 0.0)));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Dice(StdRng::seed_from_u64(2026)))
        .add_systems(Startup, spawn_wave)
        .run();
}
```

Log the seed. A player who reports a bug on "seed 2026" gives you the exact level, wave and drops.

## Choosing

```rust
use bevy::prelude::*;
use rand::{RngExt, SeedableRng, rngs::StdRng, seq::{IndexedRandom, SliceRandom}};

fn main() {
    let mut rng = StdRng::seed_from_u64(1);

    // One element.
    let names = ["Mira", "Tobias", "Ana"];
    let pick = names.choose(&mut rng).unwrap();

    // Weighted: rare items drop less often.
    let loot = [("coin", 70), ("potion", 25), ("sword", 5)];
    let (drop, _) = loot.choose_weighted(&mut rng, |item| item.1).unwrap();

    // Shuffle (a deck of cards, a random level order).
    let mut deck: Vec<u8> = (1..=10).collect();
    deck.shuffle(&mut rng);

    // A random direction on a circle.
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let direction = Vec2::from_angle(angle);
    println!("{pick} {drop} {deck:?} {direction}");
}
```

## Fair randomness

Pure random feels unfair. Streaks happen, and players remember the bad ones.

- **Shuffle bag.** Put every outcome in a bag, shuffle, deal them out, refill when empty. Over a bag you get exactly the odds you wanted, with no long droughts.
- **Pity timer.** After N misses, raise the chance until a hit, then reset.
- **Seeds per purpose.** Use one generator for loot, another for visuals. Then a particle effect that rolls more dice never changes which item dropped.

## Reproducible, even over a network

If every machine starts from the same seed and calls the generator in the **same order**, they all get the same numbers. That is how lockstep and rollback netcode ([[Multiplayer]], [[bevy_ggrs]]) stay in sync without sending the random results. It is also why gameplay randomness should not use the system clock.

## Per entity, in parallel

A `ResMut<Dice>` means only one system uses it at a time. [[bevy_rand]] puts a generator on each entity, forked from a global seed, so systems run in parallel and the whole game is still reproducible.

## On the web

The browser has no OS random source for `rand` to find. Enable the `wasm_js` feature of `getrandom` for the web build ([[Web build]]).

## Next

Make a level from random numbers: [[Procedural generation]]. Make enemies unpredictable: [[Game AI]].
