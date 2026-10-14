---
icon: monitor
tags: [rendering]
---
# Rendering

Bevy renders with `wgpu`, which talks to Vulkan, Metal, DirectX 12, OpenGL/WebGL2 and WebGPU. You never call the GPU yourself: you describe the scene with ordinary [[ECS]] components, and the renderer draws it. #rendering

## What gets drawn

| You spawn | You see |
|:--|:--|
| [`Camera2d`] or [`Camera3d`] | **nothing yet**; a camera is the eye |
| `Sprite` | a 2D image or coloured rectangle ([[Sprites]]) |
| `Mesh2d` + `MeshMaterial2d` | a 2D shape ([[2D shapes and meshes]]) |
| `Mesh3d` + `MeshMaterial3d` | a 3D object ([[Meshes and materials]]) |
| lights ([`PointLight`], [`DirectionalLight`]) | light on 3D objects ([[Lighting]]) |
| `Node`, `Text` | UI ([[UI]]) |
| `Text2d` | text in the world ([[Text 2D]]) |
| `WorldAssetRoot(gltf scene)` | a whole model ([[Models and glTF]]) |

Everything has a [`Transform`] ([[Transforms]]). No `Camera`, no picture, which is the number one "why is my screen black?" cause.

## Three steps to anything

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);                                  // 1. an eye
    commands.spawn(Sprite::from_color(Color::srgb(0.9, 0.3, 0.3), Vec2::splat(80.0))); // 2. a thing
    // 3. Transform is added for you; move it with Transform::from_xyz(...)
}
```

## The pieces, in the order to learn them

1. [[Transforms]]: position, rotation, scale.
2. [[Cameras]]: what you look through.
3. [[Colors and math]]: `Color`, `Vec3`, `Quat`.
4. [[Post-processing]]: bloom, tone mapping, anti-aliasing.
5. [[Custom materials]]: your own shaders.
6. [[Windows]]: size, title, fullscreen, cursor.

Then branch to [[2D]] or [[3D]].

## How it works (a little)

Each frame, Bevy copies what the renderer needs from your world into a separate **render world** and draws it on another thread while the next frame's logic already runs ("pipelined rendering"). That is why a heavy `Update` and a heavy draw overlap rather than add up. You only meet this if you write render plugins or custom pipelines.

> [!WARNING]
> On the web, WebGL2 has fewer features (for instance no compute shaders and limits on storage buffers). Enable the `webgpu` feature for browsers that support WebGPU, see [[Web build]].

## Next

[[Transforms]].
