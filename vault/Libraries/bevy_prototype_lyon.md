---
icon: package
tags: [lib, 2d]
bevy: 0.19
crates: bevy_prototype_lyon=0.17
---
# bevy_prototype_lyon

**Vector shapes as entities**: fills and strokes, polygons, circles, arbitrary paths (like SVG) tessellated with the `lyon` library into meshes. #lib #2d

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.17` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_prototype_lyon = "0.17"
```

## A hexagon with an outline

```rust
use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let hexagon = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(100.0),
        ..default()
    };

    commands.spawn((Camera2d, Msaa::Sample4));
    commands.spawn(
        ShapeBuilder::with(&hexagon)
            .fill(DARK_CYAN)
            .stroke((BLACK, 8.0))
            .build(),
    );
}
```

Stroked shapes look smooth thanks to `Msaa::Sample4`.

## Choosing a shapes tool

| | Entities | Fill + stroke | Paths |
|:--|:--|:--|:--|
| Bevy `Mesh2d` primitives ([[2D shapes and meshes]]) | yes | fill only | no |
| `bevy_prototype_lyon` | yes | both | yes (SVG-like) |
| [[bevy_vector_shapes]] | optional | both | no (primitives) |
| [[Gizmos]] | one frame | lines | no |

## Links

[crates.io](https://crates.io/crates/bevy_prototype_lyon) · [docs.rs](https://docs.rs/bevy_prototype_lyon) · [repo](https://github.com/Nilirad/bevy_prototype_lyon)

## Next

[[2D]], [[Libraries]].
