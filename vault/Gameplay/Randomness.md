---
icon: dices
tags: [gameplay, math]
crates: rand="0.10"
---
# Randomness

Loot drops, enemy aim, sparks, a fresh level every run: games run on random numbers. Neither `std` nor Bevy ships a generator, so add `rand`. #gameplay #math

```toml
rand = "0.10"
```

## Quick rolls

```rust
fn rolls() {
    let die: u32 = rand::random_range(1..=6);
    let angle: f32 = rand::random_range(0.0..std::f32::consts::TAU);
    let crit = rand::random_bool(0.1); // true 10% of the time
}
```

Fine for sparks. For anything that matters, own the generator and **seed** it.

## One seed, one world

Keep the generator in a resource, seeded from a number you log. The same seed gives the same numbers in the same order, so the same world. Press **Same seed again** and the rocks land exactly where they were; **New seed** throws a new field. (Build a voxel rock with the box button and it fills the field.)

```rust example=scatter
use bevy::prelude::*;
use rand::{RngExt, SeedableRng, rngs::StdRng};

#[derive(Resource)]
struct Dice(StdRng);

#[derive(Component)]
struct Rock;

fn spawn_rocks(mut commands: Commands, mut dice: ResMut<Dice>) {
    for _ in 0..12 {
        let x = dice.0.random_range(-140.0..140.0);
        let y = dice.0.random_range(-60.0..60.0);
        let size = dice.0.random_range(12.0..28.0);
        commands.spawn((
            Rock,
            Sprite::from_color(Color::srgb(0.5, 0.5, 0.55), Vec2::splat(size)),
            Transform::from_xyz(x, y, 1.0 - y / 100.0), // lower rocks in front
        ));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Dice(StdRng::seed_from_u64(2026)))
        .add_systems(Startup, spawn_rocks)
        .run();
}
```

A player who reports a bug on "seed 2026" hands you the exact level. `RngExt` is the trait that adds `random_range` and friends to every generator.

## Choosing

```rust
use bevy::prelude::*;
use rand::{SeedableRng, rngs::StdRng, seq::{IndexedRandom, SliceRandom}};

fn main() {
    let mut rng = StdRng::seed_from_u64(1);

    let names = ["Mira", "Tobias", "Ana"];
    let pick = names.choose(&mut rng).unwrap();

    // Weighted: rare items drop less often.
    let loot = [("coin", 70), ("potion", 25), ("sword", 5)];
    let (drop, _) = loot.choose_weighted(&mut rng, |item| item.1).unwrap();

    // A deck of cards, a random level order.
    let mut deck: Vec<u8> = (1..=10).collect();
    deck.shuffle(&mut rng);

    println!("{pick} {drop} {deck:?}");
}
```

## Fair beats random

Pure randomness feels unfair: streaks happen, and players remember the bad ones.

- **Shuffle bag**: put every outcome in a bag, deal them out, refill when empty. Exact odds, no droughts.
- **Pity timer**: after N misses, raise the chance until it hits.
- **One generator per purpose**: loot and particles use different ones, so extra sparks never change which item dropped.

## Good to know

- **Multiplayer**: same seed plus same order of calls means every machine rolls the same numbers. Lockstep and rollback netcode rely on it ([[bevy_ggrs]]). Never seed gameplay from the clock.
- **In parallel**: one `ResMut<Dice>` serialises the systems that use it. [[bevy_rand]] gives each entity its own generator, forked from one seed.
- **On the web**, enable `getrandom`'s `wasm_js` feature ([[Web build]]).

## Next

Grow a level from random numbers: [[Procedural generation]].
