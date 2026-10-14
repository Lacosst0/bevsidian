---
lib: true
icon: package
tags: [2d, 3d]
crates: bevy_vector_shapes=0.14
---
# bevy_vector_shapes

Draw crisp **vector shapes** (circles, lines, rectangles, arcs, polygons) with a painter, in 2D or 3D. Resolution independent, with thickness, outlines and alpha. Great for debug drawing, HUDs, procedural art and games made of shapes. #2d #3d

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_vector_shapes 0.14` supports **Bevy 0.20**.

```toml
bevy_vector_shapes = "0.14"
```

## The painter

`ShapePainter` is a system parameter you configure and draw with. The shapes last one frame (immediate mode), so draw every frame:

```rust
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Shape2dPlugin::default())   // ShapePlugin for 2D and 3D
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .add_systems(Update, draw)
        .run();
}

fn draw(mut painter: ShapePainter, time: Res<Time>) {
    painter.set_color(Color::srgb(0.95, 0.5, 0.4));
    painter.circle(60.0);                                    // at the origin

    painter.set_translation(Vec3::new(150.0, 0.0, 0.0));
    painter.set_color(Color::srgb(0.4, 0.7, 0.95));
    painter.rect(Vec2::new(80.0, 40.0));

    painter.hollow = true;
    painter.thickness = 4.0;
    painter.set_translation(Vec3::new(-150.0, 0.0, 0.0));
    painter.circle(40.0 + 10.0 * time.elapsed_secs().sin());
}
```

## Compared to other drawing options

| | Persists | Fill and stroke | Good for |
|:--|:--|:--|:--|
| [[Gizmos]] | one frame | lines only | debug |
| `Mesh2d` + `ColorMaterial` ([[2D shapes and meshes]]) | as entities | fill | real objects, collisions |
| `bevy_vector_shapes` | one frame, or retained as entities | both, thickness, outlines | HUD, effects, procedural looks |
| [[bevy_prototype_lyon]] | as entities | paths, strokes | SVG-like vector art |

It can also be used with **retained** entities (`retained` example) and **canvases** (draw into a texture).

## Links

[crates.io](https://crates.io/crates/bevy_vector_shapes) · [docs.rs](https://docs.rs/bevy_vector_shapes) · [repo](https://github.com/james-j-obrien/bevy_vector_shapes)

## Next

[[2D]].
