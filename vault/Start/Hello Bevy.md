---
icon: rocket
tags: [start]
---
# Hello Bevy

A window, a box, and the three ideas everything else builds on. Ten minutes. #start

## Install

Get Rust from [rustup.rs](https://rustup.rs), then `rustup update`: Bevy wants **1.97.1 or newer**. Linux also needs a few system libraries (Windows and macOS need nothing):

```sh
sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev
```

Fedora and others: [[Project setup#Linux libraries]].

## A new project

```sh
cargo new my_game
cd my_game
cargo add bevy
```

The first build compiles all of Bevy and takes a few minutes. Later ones take seconds, and [[Project setup]] makes them faster still.

## The smallest app

```rust ignore
use bevy::prelude::*;

fn main() {
    App::new().add_plugins(DefaultPlugins).run();
}
```

`cargo run` opens an empty window. [`App`] holds your game, [`DefaultPlugins`] brings the window, renderer, input, audio and assets, and `.run()` loops until the window closes.

## Your first system

A **system** is a plain function. You add it to a **schedule** to say when it runs.

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

- [`Startup`] runs once; [`Update`] runs every frame.
- `Commands` creates things. `Camera2d` is the eye: no camera, nothing drawn.
- `Query<&mut Transform, With<Spinner>>` reads "the position of everything that has `Spinner`".
- `Res<Time>` is shared data. `delta_secs()` is the time since the last frame, so the box turns at the same speed on every machine.

> [!TIP]
> Change the colour or spin faster (`time.delta_secs() * 3.0`) and run it again. Poking at it is the fastest way to learn.

## Watch it happen

The same idea, but you can see inside. Press play: `regenerate` runs every frame and raises the health. The arrows step one system at a time.

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

[[Project setup]] for faster builds, then [[Entities and components]].
