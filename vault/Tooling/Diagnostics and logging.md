---
icon: chart-line
tags: [tooling]
---
# Diagnostics and logging

Know what the game is doing: messages for you, numbers for tools. #tooling

## Logging

`DefaultPlugins` includes `LogPlugin`. Use the macros anywhere:

```rust
use bevy::prelude::*;

fn log_demo() {
    trace!("very detailed");
    debug!("useful while developing");
    info!("normal events");
    warn!("something looks off");
    error!("something broke");
}
```

Choose how loud it is with `RUST_LOG`:

```sh
RUST_LOG=info cargo run                       # everything at info
RUST_LOG=warn,my_game=debug cargo run         # quiet, but my_game at debug
RUST_LOG=wgpu=error,bevy_render=info cargo run
```

Or in code:

```rust
use bevy::{log::{Level, LogPlugin}, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::DEBUG,
            filter: "wgpu=error,bevy_render=info".to_string(),
            ..default()
        }))
        .run();
}
```

Also `info_once!`, `warn_once!` (print only the first time), and `error_span!` for tracing.

## Diagnostics: numbers over time

Add the plugins you want, then read the values or print them:

```rust
use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),   // FPS and frame time
            EntityCountDiagnosticsPlugin::default(), // how many entities
            LogDiagnosticsPlugin::default(),         // print them every second
        ))
        .run();
}
```

## Read a diagnostic yourself

```rust
use bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::*};

fn show_fps(diagnostics: Res<DiagnosticsStore>, mut text: Single<&mut Text>) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
    {
        text.0 = format!("{fps:.0} fps");
    }
}
```

## Your own

```rust
use bevy::{diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic}, prelude::*};

const ENEMIES: DiagnosticPath = DiagnosticPath::const_new("game/enemies");

#[derive(Component)] struct Enemy;

fn count_enemies(mut diagnostics: Diagnostics, enemies: Query<(), With<Enemy>>) {
    diagnostics.add_measurement(&ENEMIES, || enemies.iter().count() as f64);
}

fn main() {
    App::new()
        .register_diagnostic(Diagnostic::new(ENEMIES))
        .add_systems(Update, count_enemies)
        .run();
}
```

## Dev tools

Ready-made debugging helpers, available behind the `bevy_dev_tools` feature (switched on by the `dev` feature in [[Project setup]]). #tooling

### FPS overlay

```rust
use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FpsOverlayPlugin::default()))
        .run();
}
```

Shows the frame rate and a frame-time graph in a corner. Configure the font, colour and graph with `FpsOverlayConfig`. This site's own top bar has a similar FPS readout.

### State transitions

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

### Schedule data

`schedule_data` draws or prints which systems run in which schedule, and how they are ordered; compare with [[Debug crates#bevy_mod_debugdump|bevy_mod_debugdump]].

### Remote protocol

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

### Entity picking, debug UI, inspection

For a click-to-inspect experience use [[Debug crates#bevy-inspector-egui|bevy-inspector-egui]]. For hovering and debugging picking itself, see the `debug_picking` example.

## Next

[[Diagnostics and logging#Dev tools|Dev tools]].
