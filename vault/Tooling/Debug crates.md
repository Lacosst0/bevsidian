---
lib: true
icon: package
tags: [tooling]
bevy: 0.19
crates: bevy-inspector-egui=0.37; bevy_egui=0.40; bevy_mod_debugdump=0.16
---
# Debug crates

Two crates that show you what the app is doing: a live world inspector, and a picture of the schedule. See [[Debugging]].

## bevy-inspector-egui

An **inspector for your running game**: a window listing every entity and resource with all their components, editable live. Built on [[bevy_egui]] and [[Reflection]]. The single most useful debugging tool in the ecosystem. #tooling

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.37` targets **Bevy 0.19** (with `bevy_egui 0.40`). Compiled against 0.19. Bevy 0.20's own `bevy_remote` protocol ([[Diagnostics and logging#Dev tools|Dev tools]]) is the other route.

```toml
bevy-inspector-egui = "0.37"
```

### The world inspector in three lines

```rust
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}
```

Run the game: a window with a tree of entities (named by their [`Name`]), their components, and every resource. Change a value and the game reacts.

For your components to show, they must be **reflected** ([[Reflection]]): `#[derive(Reflect)]` and `#[reflect(Component)]`.

### Just one resource

```rust
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin};

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Tuning {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    friction: f32,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin::default()))
        .init_resource::<Tuning>()
        .register_type::<Tuning>()
        .add_plugins(ResourceInspectorPlugin::<Tuning>::default())
        .run();
}
```

A panel with sliders for your tuning numbers: balance a game while it runs.

### Habits that pay off

- Give entities a `Name` ([[Entities and components]]). The tree becomes readable.
- Add a resource inspector for each system's tuning values.
- Only add the plugin in debug builds: `#[cfg(debug_assertions)]`.

### Links

[crates.io](https://crates.io/crates/bevy-inspector-egui) · [docs.rs](https://docs.rs/bevy-inspector-egui) · [repo](https://github.com/jakobhellermann/bevy-inspector-egui)

## bevy_mod_debugdump

**Draws your app as a graph**: which systems are in which schedule and how they are ordered ([[Schedules and sets]]). When "why did this run before that?" has no obvious answer, look at the picture. #tooling

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.16` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_mod_debugdump = "0.16"
```

### Print a schedule as a graph

It prints Graphviz `dot` text; pipe it into `dot` to get an image:

```rust
use bevy::{log::LogPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    // No logging, so stdout holds only the graph.
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    // ... add your own plugins and systems ...
    bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
}
```

```sh
cargo run | dot -Tsvg > update.svg
```

Install Graphviz first (`apt install graphviz`, `brew install graphviz`).

### Other graphs

- The **render graph** (what the renderer does each frame).
- The **main schedule order** (First, PreUpdate, Update, ...) and plugin structure.

### Related

- Bevy's own `Stepping` (what this site's previews use) and `schedule_data` dev tool ([[Diagnostics and logging#Dev tools|Dev tools]]).
- [[Debugging]] for the rest of the debugging toolbox.

### Links

[crates.io](https://crates.io/crates/bevy_mod_debugdump) · [docs.rs](https://docs.rs/bevy_mod_debugdump) · [repo](https://github.com/jakobhellermann/bevy_mod_debugdump)
