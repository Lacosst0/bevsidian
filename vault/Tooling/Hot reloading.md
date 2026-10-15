---
parent: Debugging
icon: flame
tags: [tooling]
---
# Hot reloading

Change something, see it in the running game, no restart. #tooling

## Assets

Turn on the `file_watcher` feature (included in `dev`):

```sh
cargo run --features bevy/file_watcher
```

Now images, models, sounds and shaders reload when you save them. To react in code:

```rust
use bevy::prelude::*;

fn on_image_changed(mut events: MessageReader<AssetEvent<Image>>) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            info!("image {id} was reloaded");
        }
    }
}
```

## Data-driven design

Put game numbers in files (RON, JSON): enemy health, weapon damage, level layout. Load them as [[Custom assets]], and tune them while the game runs. This is the most productive trick in this note. `bevy_common_assets` ([[bevy_common_assets]]) makes it a few lines.

## Code

Bevy can **hot-patch systems**: you save a Rust file, Bevy swaps the changed system into the running game. Enable the `hotpatching` feature and run with the Dioxus CLI's subsecond tool:

```sh
cargo install dioxus-cli
BEVY_ASSET_ROOT="." dx serve --hot-patch --features bevy/hotpatching
```

Only the bodies of **systems** (and observers) can change this way, not type definitions. See the `hotpatching_systems` example in the Bevy repo. It is fast, but treat it as experimental.

## Shaders

WGSL/WESL files are assets, so they hot reload with `file_watcher` as well: perfect for tuning an effect ([[Custom materials]]).

## Turn it off for release

`file_watcher` and `hotpatching` belong in development builds. Do not ship them ([[Release builds]]).

## Next

[[Release builds]].
