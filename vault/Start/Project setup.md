---
icon: wrench
tags: [start]
---
# Project setup

Settings that make Bevy pleasant to work with. Copy them once, forget them. #start

## Cargo.toml

Debug builds of Bevy are slow to run. Optimize your dependencies but keep your own code quick to compile:

```toml
[package]
name = "my_game"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = "0.20"

# Your code: a little optimization, fast compiles.
[profile.dev]
opt-level = 1

# Dependencies (Bevy): full optimization.
[profile.dev.package."*"]
opt-level = 3
```

## Compile less

Bevy groups its features into **profiles**. Choose only what your game uses:

| Profile | Contains |
|:--|:--|
| `2d` | core, 2D rendering, scenes, picking |
| `3d` | core, 3D rendering, scenes, picking |
| `ui` | core, UI, scenes, picking |
| `audio` | audio playback with `.ogg` |
| `default` | all of the above |

```toml
# A 2D game with sound, no 3D code.
bevy = { version = "0.20", default-features = false, features = ["2d", "audio"] }
```

The full list is in the `bevy` crate's `docs/cargo_features.md`.

## Faster rebuilds

- **Dynamic linking**: `cargo run --features bevy/dynamic_linking`. The biggest win. Never ship with it.
- **The `dev` feature**: `cargo add bevy -F dev` turns on asset hot reloading and debug tools. Remove it for releases.
- **Linker**: since Rust 1.90, Linux uses `lld` by default. `mold` is usually not needed.
- **Nightly + Cranelift**: ~30% faster compiles of your own code, but not for web builds. [[Tooling]] has the details.

## Linux libraries

```sh
# Fedora
sudo dnf install gcc-c++ libX11-devel alsa-lib-devel systemd-devel wayland-devel libxkbcommon-devel
```

Ubuntu is in [[Hello Bevy]]. On a machine without a graphics driver, install `mesa-vulkan-drivers`.

## Assets folder

Put images, sounds and models in an `assets/` folder next to `Cargo.toml`. `asset_server.load("player.png")` reads `assets/player.png`. More in [[Assets]].

## Check it works

```sh
cargo run
```

You should see the window from [[Hello Bevy]]. If it opens and closes at once, run it in a terminal and read the log.

## Project structure

How to split a game before `main.rs` gets out of hand. #start

### One plugin per feature

Group everything about one feature in one module, with a plugin as its front door:

```
src/
  main.rs          // builds the App, adds the plugins
  player.rs        // PlayerPlugin: components, systems, observers
  enemy.rs         // EnemyPlugin
  ui/
    mod.rs         // UiPlugin
    hud.rs
    menu.rs
  state.rs         // GameState and transitions
assets/
```

```rust
// main.rs
use bevy::prelude::*;

mod player {
    use bevy::prelude::*;
    pub struct PlayerPlugin;
    impl Plugin for PlayerPlugin {
        fn build(&self, _app: &mut App) {}
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, player::PlayerPlugin))
        .run();
}
```

### Keep types near their systems

A component, the systems that use it and the plugin that registers them belong in one file. If another feature needs the component, make it `pub`. That is the whole API between features.

### Talk through events, not imports

When the player dies, `player.rs` shouldn't call into `ui.rs`. Trigger an event and let the other plugin observe it. See [[Events and observers]] and [[Messages]].

### Game states

Menu, playing, paused: model them as [[States]]. Each plugin adds its systems with `run_if(in_state(...))` or `OnEnter`.

### Libraries

If something is useful in more than one game, move it to its own crate in a Cargo workspace. Libraries should expose a plugin and avoid doing anything outside it.

## Next

[[App and plugins]], then [[Project setup#Project structure|Project structure]] when your `main.rs` gets long.
