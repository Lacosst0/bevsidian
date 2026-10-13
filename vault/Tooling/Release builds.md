---
icon: rocket
tags: [tooling, intermediate]
---
# Release builds

`cargo run` builds for development. To ship, tell Cargo to optimise for the player. #tooling #intermediate

## Profiles

Add to `Cargo.toml`. Pick **speed** (desktop) or **size** (web, mobile):

```toml
# Speed
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = "debuginfo"

# Or size
# [profile.release]
# opt-level = "z"
# lto = "fat"
# codegen-units = 1
# strip = true
```

| Setting | Does | Cost |
|:--|:--|:--|
| `opt-level` | how hard to optimise (`3` fast, `"z"` small) | compile time |
| `lto = "fat"` | whole-program optimisation (`"thin"` is a good, faster middle) | compile time, memory |
| `codegen-units = 1` | better optimisation | compile time |
| `strip` | remove symbols, smaller binary | none |
| `panic = "abort"` | smaller; a panic ends the program | no unwinding (fine for games) |

Keep your fast-iteration dev profile ([[Project setup]]) separate, and build the release with `cargo build --release`.

## Compile less

A smaller Bevy is smaller to download. Disable features you don't use (`default-features = false`, then `features = ["2d", "audio"]`), and disable unused `DefaultPlugins` members ([[App and plugins]]).

## What to ship

```text
my_game/
  my_game(.exe)     ← target/release/my_game
  assets/           ← your assets folder, next to the binary
```

Bevy looks for `assets/` next to the executable. Set `BEVY_ASSET_ROOT` or `AssetPlugin { file_path: ... }` to change it ([[Assets]]).

## Per platform

| Platform | Notes |
|:--|:--|
| Windows | build on Windows (MSVC); add an icon with `winresource`; ship the `assets` folder |
| macOS | create a `.app` bundle with `cargo-bundle`; sign and notarise to avoid warnings |
| Linux | an AppImage/Flatpak is friendliest; ship the binary + assets |
| Web | see [[Web build]] |
| Android / iOS | Bevy supports both; see the repo's `mobile` example and Bevy CLI docs |

## Cross-platform CI

Use GitHub Actions to build each platform on a tag, like this repository's `release.yml`, and attach the zips to a release. `cargo-dist` automates installers.

## Before you ship

- Remove `file_watcher`, `hotpatching`, `dynamic_linking`, `dev` ([[Hot reloading]]).
- Run once with `RUST_LOG=warn` and fix every warning.
- Test the release build on a machine without Rust installed.
- Profile if it is slow: [[Profiling]].

## Next

[[Profiling]], then [[Web build]].
