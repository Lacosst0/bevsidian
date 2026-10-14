---
icon: mountain
tags: [gameplay, math]
crates: rand="0.10"
---
# Procedural generation

Instead of drawing every level, write **rules** and let random numbers fill in the rest. A few lines make endless caves, islands, dungeons and forests. #gameplay #math

> [!TIP]
> Two rules make procedural content good. **Seed it** ([[Randomness]]), so a level can be reproduced. **Generate data first, entities second**: build a plain `Vec` or grid, check it, then spawn sprites from it.

## Caves: cellular automata

Start with static: every cell is a wall with about 46% chance. Then repeat **one rule** a few times: *a cell becomes a wall if five or more of its eight neighbours are walls, otherwise floor*. Isolated specks vanish, crowds join up, and noise melts into caves.

First the data and the random fill:

```rust example=cave
use bevy::prelude::*;
use rand::{RngExt, SeedableRng, rngs::StdRng};

const WIDTH: usize = 40;
const HEIGHT: usize = 20;

#[derive(Resource)]
struct Dice(StdRng);

#[derive(Resource)]
struct Cave {
    walls: Vec<bool>,   // WIDTH * HEIGHT, row by row; true = wall
    steps: u32,
    wait: f32,
}

fn random_fill(mut cave: ResMut<Cave>, mut dice: ResMut<Dice>) {
    for wall in &mut cave.walls {
        *wall = dice.0.random_bool(0.46);
    }
    cave.steps = 0;
}
```

Run it. That is noise. Now the rule:

```rust example=cave
impl Cave {
    /// Walls among the eight neighbours; outside the map counts as wall.
    fn wall_count(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for dy in -1..=1_i32 {
            for dx in -1..=1_i32 {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                let outside = nx < 0 || ny < 0 || nx >= WIDTH as i32 || ny >= HEIGHT as i32;
                if (dx, dy) != (0, 0) && (outside || self.walls[ny as usize * WIDTH + nx as usize]) {
                    count += 1;
                }
            }
        }
        count
    }

    fn smooth(&mut self) {
        let next: Vec<bool> = (0..WIDTH * HEIGHT)
            .map(|i| self.wall_count(i % WIDTH, i / WIDTH) >= 5)
            .collect();
        self.walls = next;
        self.steps += 1;
    }
}

/// One smoothing step every half second, five in all, so you can watch it.
fn smooth(time: Res<Time>, mut cave: ResMut<Cave>) {
    cave.wait += time.delta_secs();
    if cave.wait >= 0.5 && cave.steps < 5 {
        cave.wait = 0.0;
        cave.smooth();
    }
}
```

Two details matter. `smooth` builds a **new** map (`next`) and swaps it in. Updating in place would let early cells change what later cells see, and the result would lean toward one corner. And the map edge counts as wall, so the cave is sealed.

Press **New cave** for another seed. Change `0.46` to `0.40` (airy, many rooms) or `0.52` (tight tunnels), and `>= 5` to `>= 4` (blobbier).

> [!NOTE]
> The preview uses a small generator of its own so that it needs no crates; the code above uses `rand` and does the same.

## Random walk: the drunken digger

For a winding tunnel, send a "digger" wandering and carve where it walks:

```rust
use bevy::prelude::*;
use rand::{RngExt, SeedableRng, rngs::StdRng};

fn dig(width: i32, height: i32, steps: usize, rng: &mut StdRng) -> Vec<IVec2> {
    let mut at = IVec2::new(width / 2, height / 2);
    let mut floor = vec![at];
    for _ in 0..steps {
        let step = [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y][rng.random_range(0..4)];
        at = (at + step).clamp(IVec2::ONE, IVec2::new(width - 2, height - 2));
        if !floor.contains(&at) {
            floor.push(at);
        }
    }
    floor
}
```

Everything it visits is floor, so every room is **connected by construction**. That is the big weakness of the cave method above: it can leave sealed pockets. Fix a cave by flood filling from a floor cell and keeping only the cells it reaches ([[Pathfinding]] is that same walk).

## Rooms and corridors

Dungeons: scatter rectangles, drop any that overlap, and join the centres of neighbours with L-shaped corridors. The pieces are `IVec2`s and a `Rect`-like struct; the only trick is the order: **rooms first, corridors second, walls last** (a wall is just "any empty cell next to a floor").

## Noise: smooth randomness

Pure random jumps between neighbours. **Noise** is random that changes smoothly, which makes hills, clouds and rivers. The idea of *value noise*: random values at a coarse grid, blended between:

```rust
use bevy::prelude::*;

/// A pseudo-random number in 0..1 for a grid point; same input, same output.
fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263) ^ seed;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

/// Smooth in between: bilinear blend of the four surrounding grid values.
fn value_noise(p: Vec2, seed: u32) -> f32 {
    let (cell, f) = (p.floor(), p.fract());
    let (x, y) = (cell.x as i32, cell.y as i32);
    let t = f * f * (Vec2::splat(3.0) - 2.0 * f);       // smoothstep: no visible grid
    let top = hash(x, y, seed).lerp(hash(x + 1, y, seed), t.x);
    let bottom = hash(x, y + 1, seed).lerp(hash(x + 1, y + 1, seed), t.x);
    top.lerp(bottom, t.y)
}

/// Layers of noise at doubling frequency: big shapes plus fine detail.
fn height(p: Vec2, seed: u32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    for _ in 0..4 {
        total += value_noise(p * frequency, seed) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    total
}

/// Height to a tile: water, sand, grass, rock.
fn terrain(h: f32) -> &'static str {
    match h {
        h if h < 0.35 => "water",
        h if h < 0.42 => "sand",
        h if h < 0.65 => "grass",
        _ => "rock",
    }
}
```

`height(cell as Vec2 * 0.1, seed)` per cell, then `terrain`. Smaller multiplier, bigger continents. The layered sum is called **fractal noise** (fBm). Libraries offer faster, better noise (Perlin, simplex, Worley): the `noise` crate is old but works, and `fastnoise-lite` and `bracket-noise` are alternatives. For 3D terrain, make a mesh from the heights ([[Meshes and materials]]).

## Wave function collapse

Give the generator **tiles and which tiles may touch**, and it fills the map like a Sudoku: pick the cell with the fewest options, choose one, propagate. It reproduces the *style* of a hand-made example, so it is popular for towns and dungeons. Worth a library rather than writing it yourself.

## Spawning from the data

When the grid is ready, turn it into entities in one go. BSN fits well: a `Vec` of scenes ([[Composing scenes]]), or use a [[Tilemaps]] chunk for thousands of tiles.

```rust
use bevy::prelude::*;

fn spawn_cave(mut commands: Commands, walls: &[bool], width: usize) {
    for (i, wall) in walls.iter().enumerate() {
        let (x, y) = ((i % width) as f32, (i / width) as f32);
        let color = if *wall { Color::srgb(0.3, 0.32, 0.42) } else { Color::srgb(0.1, 0.1, 0.15) };
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(8.0)),
            Transform::from_xyz(x * 8.0, y * 8.0, 0.0),
        ));
    }
}
```

## Make it playable

- **Check it.** After generating, test that the start can reach the exit ([[Pathfinding]]). If not, re-roll with the next seed.
- **Place things by rule.** Enemies far from the start, treasure in dead ends, one key before its door.
- **Keep the seed.** Show it on the game-over screen.
- **Tune with a preview.** The fastest way to find good numbers is the sliders-and-regenerate loop you just used.

## Next

[[Randomness]] has the tools; [[Game AI]] populates what you built.
