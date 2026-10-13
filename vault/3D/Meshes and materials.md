---
icon: cuboid
tags: [3d, beginner]
---
# Meshes and materials

A 3D object is a **mesh** (its shape) plus a **material** (its surface). Both are [[Assets|assets]] on an entity. #3d #beginner

## A cube, the classic way

```rust
use bevy::prelude::*;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.6, 0.4, 0.9))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}
```

`meshes.add(...)` stores the shape and returns a handle; `Mesh3d` and `MeshMaterial3d` hold the handles. The BSN version of the same scene is in [[Scenes and BSN]].

## Shapes you get for free

| Shape | Example |
|:--|:--|
| `Cuboid` | `Cuboid::new(w, h, d)` |
| `Sphere` | `Sphere::new(r)`; `.mesh().ico(5)` or `.mesh().uv(32, 18)` for smoothness |
| `Plane3d` | `Plane3d::default().mesh().size(10.0, 10.0)` for a floor |
| `Cylinder`, `Cone`, `Capsule3d`, `Torus` | `Cylinder::new(radius, height)` ... |

## Materials: StandardMaterial

`StandardMaterial` is physically based:

```rust
use bevy::prelude::*;

fn materials(mut materials: ResMut<Assets<StandardMaterial>>, asset_server: Res<AssetServer>) {
    let _gold = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.77, 0.34),
        metallic: 1.0,                  // 0 = plastic/wood, 1 = metal
        perceptual_roughness: 0.2,      // 0 = mirror, 1 = matte
        ..default()
    });

    let _glow = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: LinearRgba::rgb(0.0, 8.0, 4.0),   // light the surface gives off (use with bloom)
        ..default()
    });

    let _brick = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("brick_albedo.png")),
        normal_map_texture: Some(asset_server.load("brick_normal.png")),
        perceptual_roughness: 0.9,
        ..default()
    });

    let _glass = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.8, 1.0, 0.3),
        alpha_mode: AlphaMode::Blend,   // see-through
        ..default()
    });
}
```

| Field | What it changes |
|:--|:--|
| `base_color` / `base_color_texture` | the colour of the surface |
| `metallic`, `perceptual_roughness` | how it reflects light |
| `emissive` | self-lighting |
| `normal_map_texture` | fake small bumps |
| `alpha_mode` | `Opaque`, `Mask(0.5)` (cut-out), `Blend` (glass), `Add` |
| `unlit` | ignore lights (UI-like or retro look) |

## Reuse handles

A thousand identical trees need **one** mesh and **one** material:

```rust
use bevy::prelude::*;

fn forest(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cone::new(0.5, 2.0));
    let material = materials.add(Color::srgb(0.1, 0.5, 0.2));
    for i in 0..100 {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz((i % 10) as f32 * 2.0, 1.0, (i / 10) as f32 * 2.0),
        ));
    }
}
```

Bevy then draws them together in one call.

## Your own mesh

```rust
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

fn quad() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[-1.0, 0.0, -1.0], [1.0, 0.0, -1.0], [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]],
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 4])
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        )
        .with_inserted_indices(Indices::U32(vec![0, 3, 2, 0, 2, 1]))
}
```

Positions, **normals** (for lighting) and **UVs** (for textures) are the three things almost every mesh has. Triangle order (winding) decides which side is "front".

## Custom looks

When `StandardMaterial` is not enough, extend it or write a shader: [[Custom materials]].

## Next

[[Lighting]]: without light, none of this is visible.
