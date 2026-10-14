---
icon: smartphone
tags: [tooling]
check: skip
---
# Mobile builds

Bevy runs on Android and iOS. The same game code, a different project wrapper around it. This note is the map; the exact commands change with each tool release, so follow the official example for the version you use. #tooling

> [!WARNING]
> Mobile has more moving parts than desktop or web: SDKs, signing, a device or emulator. Get the game running well on desktop and in the browser ([[Web build]]) first, and test on a phone early, not at the end.

## Make the game a library

Mobile apps start from native code in a library, not from `main`. Split your crate into a library with the game and a thin `main.rs`:

```rust
// src/lib.rs
use bevy::prelude::*;

#[bevy_main]
pub fn main() {
    App::new().add_plugins(DefaultPlugins).run();
}
```

```toml
# Cargo.toml
[lib]
crate-type = ["lib", "cdylib"]

[dependencies]
bevy = { version = "0.20", features = ["android-game-activity"] }
```

`#[bevy_main]` makes the same `main` work on every platform.

## Android

Tools: the Android SDK and NDK, the Rust target, and `cargo-ndk`.

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
cargo ndk -t arm64-v8a -P 26 -o android/app/src/main/jniLibs build
```

That builds the native library into a small Gradle project (`./gradlew build` or Android Studio) which packs it into an APK.

- **Activity type.** Bevy uses `GameActivity` by default (Android 12, API 31, and up). For older phones use `android-native-activity` instead.
- **Audio** needs API 26 or higher.
- **Logs.** `adb logcat | grep 'RustStdoutStderr\|bevy\|wgpu'`
- Bevy's repository has the complete working project in `examples/mobile`. Copy it rather than starting from nothing.

## iOS

You need a Mac with Xcode, and the Rust targets:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

The example project in `examples/mobile` has a Makefile and an Xcode project that build, install and run the app in the simulator (`make run`). Real devices need an Apple developer account for signing.

## What changes in your game

| Topic | On a phone |
|:--|:--|
| input | no keyboard or mouse: use [[Gamepad and touch#Touch\|Touch]] and on-screen controls; draw big touch targets |
| screen | notches and rounded corners: keep UI away from the edges ("safe area") |
| orientation | decide portrait or landscape and set it in the project |
| size | the screen is small: raise UI scale ([[Accessibility and localization]]) |
| lifecycle | the OS can pause or kill the app at any time: save often ([[Saving and loading]]) |
| power | battery and heat: insert `WinitSettings::mobile()`, which sleeps between frames until something happens |
| size of the app | release profile with `opt-level = "z"`, `lto`, `strip` ([[Release builds]]) |
| assets | smaller textures; compressed formats; stores limit download size |

## Shipping

Both stores want a signed, release build, an icon set, screenshots and a privacy policy. Plan a day for the first submission. Tools such as `cargo-mobile2` can generate and manage both projects from one config if you prefer that to hand-managed Gradle and Xcode projects.

## Next

[[Gamepad and touch#Touch|Touch]] for input, [[Release builds]] for size, [[Ship it]] for the whole path.
