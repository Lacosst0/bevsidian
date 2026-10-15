---
icon: wrench
tags: [start]
---
# Project setup

Settings that make Bevy pleasant to work with. Copy them once and forget them. #start

## Cargo.toml

Bevy built in debug mode runs slowly. Optimise your dependencies fully and your own code a little, so it still compiles fast:

```toml
[package]
name = "my_game"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = "0.20"

[profile.dev]
opt-level = 1          # your code: quick to compile

[profile.dev.package."*"]
opt-level = 3          # Bevy and friends: fast to run
```

## Compile less

Bevy groups its features into **profiles**. Turn on only what the game uses:

| Profile | Contains |
|:--|:--|
| `2d` | core, 2D rendering, scenes, picking |
| `3d` | core, 3D rendering, scenes, picking |
| `ui` | core, UI, scenes, picking |
| `audio` | playback of `.ogg` files |
| `default` | all of the above |

```toml
bevy = { version = "0.20", default-features = false, features = ["2d", "audio"] }
```

The full list lives in the `bevy` crate's `docs/cargo_features.md`.

## Faster rebuilds

- **Dynamic linking**: `cargo run --features bevy/dynamic_linking`. The biggest win; never ship with it.
- **`dev` feature**: `cargo add bevy -F dev` adds asset hot reloading and debug tools. Drop it for releases.
- **Linker**: Linux already links with `lld`[^lld], so `mold` rarely helps.
- **Nightly + Cranelift**: about 30% faster builds of your own code (not on the web). See [[Tooling]].

## Linux libraries

```sh
# Ubuntu / Debian
sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev
# Fedora
sudo dnf install gcc-c++ libX11-devel alsa-lib-devel systemd-devel wayland-devel libxkbcommon-devel
```

No graphics driver (a VM, a server)? Install `mesa-vulkan-drivers`.

## Assets

Images, sounds and models go in `assets/` next to `Cargo.toml`: `asset_server.load("player.png")` reads `assets/player.png`. More in [[Assets]].

## Project structure

Split the game before `main.rs` gets out of hand: **one plugin per feature**, each in its own module.

```
src/
  main.rs        // builds the App, adds the plugins
  player.rs      // PlayerPlugin: components, systems, observers
  enemy.rs       // EnemyPlugin
  ui/            // UiPlugin: hud.rs, menu.rs
  state.rs       // GameState and its transitions
assets/
```

```rust
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

- **Keep types near their systems.** A component, the systems that use it and the plugin that adds them share a file; make it `pub` only when another feature needs it.
- **Talk through events, not imports.** When the player dies, `player.rs` triggers an event and the UI observes it ([[Events and observers]], [[Messages]]).
- **Model menus and pauses as [[States]]**, and gate each plugin's systems with `run_if(in_state(...))`.
- **Reusable across games?** Move it into its own crate in a workspace, exposing one plugin.

## Next

[[App and plugins]].

[^lld]: The default for `x86_64-unknown-linux-gnu` since Rust 1.90.
