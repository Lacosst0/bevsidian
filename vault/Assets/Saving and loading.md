---
icon: archive
tags: [assets, advanced]
---
# Saving and loading

Bevy can write part of the world to a file and read it back. It works through [[Reflection]]: every component you want saved must be reflectable. #assets #advanced

> [!NOTE]
> Bevy 0.20 calls this *world serialization*: `DynamicWorld` (the saved data) and `DynamicWorldRoot` (spawns it). It is not the same as BSN scenes ([[Scenes and BSN]]), which are written in code.

## Make components saveable

```rust
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Score {
    points: u32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Position {
    x: f32,
    y: f32,
}
```

Derive `Reflect`, add `#[reflect(Component)]`, and make sure the type is registered (`app.register_type::<Score>()`, usually automatic).

## Save

Build a separate world containing only what you want to keep, turn it into a `DynamicWorld`, and write it out as text:

```rust
use bevy::prelude::*;
use std::{fs::File, io::Write};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Score { points: u32 }

fn save(world: &mut World) {
    let registry = world.resource::<AppTypeRegistry>().clone();

    let mut save = World::new();
    save.spawn(Score { points: 120 });

    let dynamic = DynamicWorld::from_world_with(&save, &registry.read());
    let text = dynamic.serialize(&registry.read()).unwrap();

    File::create("assets/save.scn.ron")
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .expect("could not write the save");
}
```

(On native only. The browser has no files; use `bevy_pkv` from [[Libraries]].)

## Load

Spawn a `DynamicWorldRoot` with the saved asset:

```rust
use bevy::prelude::*;

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DynamicWorldRoot(asset_server.load("save.scn.ron")));
}
```

The entities appear as children of that root. For a glTF or model scene use `WorldAssetRoot(asset_server.load("model.gltf#Scene0"))`.

## What to save

- Save **data**, not state of the engine: score, inventory, position, level number.
- Keep the saved components small and stable. Add a version number to your save so you can migrate it.
- For a few settings a `Resource` + `serde` + a JSON file is simpler. Or use `bevy_pkv` ([[Libraries]]): a key-value store that works on desktop and web.

## Next

[[Rendering]]: getting the world onto the screen.
