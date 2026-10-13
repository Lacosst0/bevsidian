---
icon: package
tags: [3d, beginner]
---
# Models and glTF

You do not hand-write most 3D art. Make it in Blender (or buy it), export **glTF**, and load it. #3d #beginner

## Load a model

glTF (`.glb` is the single-file version) is Bevy's main 3D format. One file holds meshes, materials, textures, animations and whole scenes. Parts have **labels**:

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb"))),
        Transform::from_scale(Vec3::splat(0.02)),
    ));
}
```

| Label | Gives |
|:--|:--|
| `GltfAssetLabel::Scene(0)` | the first scene: a tree of entities with meshes and materials |
| `GltfAssetLabel::Mesh(0).from_asset(..)` | just one mesh |
| `GltfAssetLabel::Animation(2).from_asset(..)` | one animation clip |
| `GltfAssetLabel::Material(0).from_asset(..)` | one material |

`WorldAssetRoot` spawns the scene as **children** of that entity. Move, rotate or scale the root and the whole model follows ([[Relationships and hierarchy]]).

## When is it ready?

The scene spawns a moment after loading. React with `WorldInstanceReady`:

```rust
use bevy::{prelude::*, world_serialization::WorldInstanceReady};

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/car.glb"))))
        .observe(on_ready);
}

fn on_ready(ready: On<WorldInstanceReady>, children: Query<&Children>, names: Query<&Name>) {
    for descendant in children.iter_descendants(ready.entity) {
        if let Ok(name) = names.get(descendant) {
            info!("model part: {name}");
        }
    }
}
```

Node names from the model show up as [`Name`] components. Use them to find the wheel, the muzzle or the hand slot, then attach things there or drive them with a system.

## Exporting from Blender

- File → Export → glTF 2.0 (`.glb`).
- Apply transforms and modifiers; use "+Y Up".
- Keep textures embedded (`.glb`), or beside the file (`.gltf`).
- Name important nodes: `Muzzle`, `Door`, `Wheel_FL`.
- One animation per clip; name them (`Idle`, `Run`).

## Scale and orientation

Many models are huge or tiny (a fox is 100 units tall). Fix on the root with `Transform::from_scale`. A model that faces the wrong way: rotate the root, never the children.

## Collision from a model

Visual meshes are usually too detailed for physics. Add simple colliders (box, capsule) by hand, or generate from the mesh with your physics library ([[Physics]]).

## Level design tools

For whole levels, [[bevy_trenchbroom]] (Quake-style brushes, TrenchBroom editor) and [[bevy_skein]] (Blender → Bevy components) put gameplay data inside the model file.

## Next

[[3D animation]].
