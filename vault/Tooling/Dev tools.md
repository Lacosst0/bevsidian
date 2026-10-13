---
icon: terminal
tags: [tooling, beginner]
---
# Dev tools

Ready-made debugging helpers, available behind the `bevy_dev_tools` feature (switched on by the `dev` feature in [[Project setup]]). #tooling #beginner

## FPS overlay

```rust
use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FpsOverlayPlugin::default()))
        .run();
}
```

Shows the frame rate and a frame-time graph in a corner. Configure the font, colour and graph with `FpsOverlayConfig`. This site's own top bar has a similar FPS readout.

## State transitions

Print every change of a [[States|state]]:

```rust
use bevy::{dev_tools::states::log_transitions, prelude::*};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Mode { #[default] Menu, Playing }

fn main() {
    App::new()
        .init_state::<Mode>()
        .add_systems(Update, log_transitions::<Mode>)
        .run();
}
```

## Schedule data

`schedule_data` draws or prints which systems run in which schedule, and how they are ordered; compare with [[bevy_mod_debugdump]].

## Remote protocol

With the `bevy_remote` Cargo feature a running game answers JSON-RPC requests: list entities, read and write components, from a script or an external editor. It is the base for editors and inspectors that run in another process.

```rust
use bevy::{prelude::*, remote::{http::RemoteHttpPlugin, RemotePlugin}};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RemotePlugin::default(), RemoteHttpPlugin::default()))
        .run();
}
```

Components must be reflected ([[Reflection]]) to be visible.

## Entity picking, debug UI, inspection

For a click-to-inspect experience use [[bevy-inspector-egui]]. For hovering and debugging picking itself, see the `debug_picking` example.

## Next

[[Debugging]].
