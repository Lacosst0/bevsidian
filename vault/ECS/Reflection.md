---
icon: text-select
tags: [ecs, advanced]
---
# Reflection

Reflection lets Bevy look at your types **at runtime**: list the fields, read a value by name, build one from a file. Scenes, inspectors and the entity cards on this page rely on it. #ecs #advanced

## Derive it

```rust
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
struct Stats {
    strength: u32,
    name: String,
}

fn main() {
    App::new()
        .register_type::<Stats>()   // put it in the type registry
        .run();
}
```

- `Reflect` describes the type.
- `#[reflect(Component)]` records "this can be used as a component", so tools can add it to entities by name.
- `register_type` adds it to the registry. With the default `reflect_auto_register` feature, derived types register themselves, so you only need `register_type` for generic types or on platforms where auto registration is unavailable.

## Look inside

```rust
use bevy::prelude::*;
use bevy::reflect::structs::Struct;

#[derive(Reflect)]
struct Stats { strength: u32, speed: f32 }

fn inspect() {
    let stats = Stats { strength: 3, speed: 1.5 };
    for (index, value) in stats.iter_fields().enumerate() {
        let name = stats.name_at(index).unwrap();
        info!("{name} = {value:?}");
    }

    // Change a field by name.
    let mut stats = stats;
    *stats.field_mut("strength").unwrap().try_downcast_mut::<u32>().unwrap() = 10;
}
```

## Why it matters

- **Inspectors** like `bevy-inspector-egui` show and edit any reflected component ([[Debugging]]).
- **Scenes** (see [[Saving and loading]]) are written and read through reflection.
- **Remote tools** (`bevy_remote`) query a running game.
- **Modding** and scripting: create components from names.

## Reflect your components by default

If a component might ever be saved, inspected or shown in an editor, derive `Reflect` and `#[reflect(Component)]` on it. It costs nothing at runtime.

## Next

[[Testing]].
