---
lib: true
parent: Debug crates
icon: package
tags: [tooling]
bevy: 0.19
crates: bevy-inspector-egui=0.37; bevy_egui=0.40
---
# bevy-inspector-egui

An **inspector for your running game**: a window listing every entity and resource with all their components, editable live. Built on [[bevy_egui]] and [[Reflection]]. The single most useful debugging tool in the ecosystem. #tooling

#0.19 · on #0.20, Bevy's `bevy_remote` protocol ([[Diagnostics and logging#Dev tools|Dev tools]]) is the other route

```toml
bevy-inspector-egui = "0.37"
```

## The world inspector in three lines

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

## Just one resource

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

## Habits that pay off

- Give entities a `Name` ([[Entities and components]]). The tree becomes readable.
- Add a resource inspector for each system's tuning values.
- Only add the plugin in debug builds: `#[cfg(debug_assertions)]`.

## Links

[crates.io](https://crates.io/crates/bevy-inspector-egui) · [docs.rs](https://docs.rs/bevy-inspector-egui) · [repo](https://github.com/jakobhellermann/bevy-inspector-egui)

## Next

[[Debugging]].
