---
icon: wrench
tags: [start, beginner]
---
# Project setup

Settings that make Bevy pleasant to work with. Copy them once, forget them. #start #beginner

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

Bevy 0.20 groups its features into **profiles**. Choose only what your game uses:

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

## Next

[[App and plugins]], then [[Project structure]] when your `main.rs` gets long.
