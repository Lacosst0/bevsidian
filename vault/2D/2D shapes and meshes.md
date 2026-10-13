---
icon: shapes
tags: [2d, intermediate]
---
# 2D shapes and meshes

Sprites are pictures. When you want a circle, a polygon or your own geometry, draw a **mesh**. #2d #intermediate

## A mesh and a material

Two components, each pointing at an asset: [`Mesh2d`] (the shape) and [`MeshMaterial2d`] (how it is coloured).

```rust
use bevy::prelude::*;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.9, 0.4, 0.3))),
        Transform::from_xyz(-120.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 60.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.7, 0.9))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(50.0, 6))),     // hexagon
        MeshMaterial2d(materials.add(Color::srgb(0.6, 0.9, 0.4))),
        Transform::from_xyz(120.0, 0.0, 0.0),
    ));
}
```

Meshes and materials are [[Assets|assets]]: `add` stores them and returns a handle. **Reuse handles** for many identical objects: one mesh, one material, a hundred entities.

## Primitive shapes

| 2D shape | Makes |
|:--|:--|
| `Circle::new(radius)` | circle |
| `Rectangle::new(w, h)` | rectangle |
| `Ellipse::new(half_w, half_h)` | ellipse |
| `RegularPolygon::new(radius, sides)` | triangle, hexagon... |
| `Capsule2d::new(radius, length)` | pill |
| `Annulus::new(inner, outer)` | ring |
| `Triangle2d::new(a, b, c)` | any triangle |

The same shapes also give geometry for collisions and [[Colors and math|bounding boxes]].

## Change the colour at runtime

```rust
use bevy::prelude::*;

#[derive(Component)] struct Pulse;

fn pulse(
    time: Res<Time>,
    query: Query<&MeshMaterial2d<ColorMaterial>, With<Pulse>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for handle in &query {
        if let Some(mut material) = materials.get_mut(&handle.0) {
            let t = time.elapsed_secs().sin() * 0.5 + 0.5;
            material.color = Color::srgb(t, 0.4, 1.0 - t);
        }
    }
}
```

Remember: this edits the *asset*, so every entity sharing that material changes. Give each entity its own material if they should differ.

## Your own mesh

A mesh is a list of vertices and triangles:

```rust
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

fn triangle() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 60.0, 0.0], [-50.0, -30.0, 0.0], [50.0, -30.0, 0.0]],
        )
        .with_inserted_indices(Indices::U32(vec![0, 1, 2]))
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(triangle())),
        MeshMaterial2d(materials.add(Color::WHITE)),
    ));
}
```

Add `Mesh::ATTRIBUTE_COLOR` for a colour per vertex.

## Textures and shaders on meshes

`ColorMaterial { texture: Some(handle), ..default() }` puts an image on the shape. For effects write your own: [[Custom materials]].

## Sprites or meshes?

| Use a sprite when | Use a mesh when |
|:--|:--|
| you have a picture | you need a non-rectangular shape |
| you animate frames | you want a custom shader |
| you want sliced or tiled images | you generate geometry |

## Lines and debug drawing

For quick outlines, arrows and grids use gizmos ([[Gizmos]]). For vector art with strokes, see [[bevy_prototype_lyon]].

## Next

[[Text 2D]].
