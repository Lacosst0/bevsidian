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

## Next

[[Dev tools]].
