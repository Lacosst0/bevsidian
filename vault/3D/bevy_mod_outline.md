---
lib: true
parent: 3D effect crates
icon: package
tags: [3d]
check: skip
---
# bevy_mod_outline

**Outlines around meshes**: the glowing edge on a selected unit, the border on an interactable. Uses vertex extrusion or a jump-flood algorithm; supports opaque and transparent outlines, render layers, skinned meshes. #3d

#0.19

```toml
bevy_mod_outline = "0.13"
```

## Idea

Add the plugin, then add an outline component to any mesh entity. Toggle it on hover or selection ([[Picking]]):

```rust ignore
// Sketch, see the crate's `shapes` example for the exact fields of your version.
app.add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin::default()));

commands.spawn((
    Mesh3d(mesh),
    MeshMaterial3d(material),
    OutlineVolume { visible: true, width: 4.0, colour: Color::WHITE },
));
```

Hook `PointerOver` / `PointerOut` observers to flip `visible` and you have hover highlights for 3D objects.

## Alternatives

- A second slightly-scaled, back-face-only mesh (cheap and DIY).
- Post-process edge detection ([[Post-processing]]).

## Links

[crates.io](https://crates.io/crates/bevy_mod_outline) · [docs.rs](https://docs.rs/bevy_mod_outline) · [repo](https://github.com/komadori/bevy_mod_outline)

## Next

[[3D]].
