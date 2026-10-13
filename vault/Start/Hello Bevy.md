---
icon: rocket
tags: [start, beginner]
---
# Hello Bevy

Make a window, put something in it, and see how the pieces fit. About ten minutes. #start #beginner

## Install

You need Rust (get it from [rustup.rs](https://rustup.rs)). Bevy 0.20 needs a recent stable compiler: **1.97.1 or newer**, so run `rustup update`.

On Linux you also need a few system libraries. On Ubuntu:

```sh
sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev
```

Windows and macOS need nothing extra. More distros: [[Project setup]].

## A new project

```sh
cargo new my_game
cd my_game
cargo add bevy
```

The first build compiles all of Bevy and takes a few minutes. After that it is fast; [[Project setup]] makes it faster still.

## The smallest app

```rust ignore
use bevy::prelude::*;

fn main() {
    App::new().add_plugins(DefaultPlugins).run();
}
```

`cargo run` opens an empty window. Three ideas are already here:

- [`App`] holds your game: its data, its systems, its plugins.
- [`DefaultPlugins`] adds what almost every game needs: a window, rendering, input, audio, assets.
- `.run()` starts the loop and does not return until the window closes.

## Your first system

A *system* is a plain function. Register it in a *schedule* to say when it runs.

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, spin)
        .run();
}

#[derive(Component)]
struct Spinner;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Spinner,
        Sprite::from_color(Color::srgb(0.9, 0.4, 0.2), Vec2::splat(100.0)),
    ));
}

fn spin(time: Res<Time>, mut spinners: Query<&mut Transform, With<Spinner>>) {
    for mut transform in &mut spinners {
        transform.rotate_z(time.delta_secs());
    }
}
```

- [`Startup`] runs once, [`Update`] runs every frame.
- `Commands` creates things. `Camera2d` is the eye; without a camera nothing is drawn.
- `Query<&mut Transform, With<Spinner>>` means "give me the position of every entity that has `Spinner`".
- `Res<Time>` is global data. `delta_secs()` is the time since the last frame, so the box turns at the same speed on every computer.

> [!TIP]
> Change the colour, or spin faster with `time.delta_secs() * 3.0`, then run it again. Breaking things is how you learn.

## Watch it happen

The same idea with an entity you can see. Press play: `regenerate` runs every frame and raises the health; step through it with the arrows.

```rust example=ecs fps=6
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, Health(50), Transform::default()));
}

fn regenerate(time: Res<Time>, mut query: Query<&mut Health, With<Player>>) {
    for mut health in &mut query {
        health.0 = (health.0 + 1).min(100);
    }
}
```

## Next

1. [[Project setup]]: faster builds and the right features.
2. [[App and plugins]]: how an app is put together.
3. [[Entities and components]]: the data model, properly.
