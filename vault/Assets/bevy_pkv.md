---
lib: true
icon: package
tags: [assets, tooling]
bevy: 0.19
crates: bevy_pkv=0.16; serde={ version = "1", features = ["derive"] }
---
# bevy_pkv

A tiny **persistent key-value store** that works on desktop *and* in the browser (it uses local storage there). Perfect for settings, high scores, and "continue" data. #tooling

#0.19

```toml
bevy_pkv = "0.16"
serde = { version = "1", features = ["derive"] }
```

## Store and read

```rust
use bevy::prelude::*;
use bevy_pkv::PkvStore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Profile {
    name: String,
    best_score: u32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(PkvStore::new("MyStudio", "MyGame"))   // names the save location
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut pkv: ResMut<PkvStore>) {
    if let Ok(profile) = pkv.get::<Profile>("profile") {
        info!("welcome back {} (best {})", profile.name, profile.best_score);
    } else {
        let profile = Profile { name: "Ada".into(), best_score: 0 };
        pkv.set("profile", &profile).expect("could not save");
    }
}
```

Simple strings are even simpler: `pkv.set_string("lang", "en")` and `pkv.get::<String>("lang")`.

## Where it saves

Native: your OS's per-user data folder. Web: the browser's local storage. That makes it the right answer to "how do I save on the web?" ([[Web build]], [[Saving and loading]]).

## Pair with

Settings structs from [[Resources]], loaded at startup and written when changed ([[Change detection]]).

## Links

[crates.io](https://crates.io/crates/bevy_pkv) · [docs.rs](https://docs.rs/bevy_pkv) · [repo](https://github.com/johanhelsing/bevy_pkv)

## Next

[[Assets]], [[Saving and loading]].
