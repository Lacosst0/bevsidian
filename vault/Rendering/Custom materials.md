---
icon: wand-sparkles
tags: [rendering]
---
# Custom materials

Write your own look with a shader. Bevy shaders are written in WGSL (or WESL, an extension that adds `import`). #rendering

## The idea

A **material** is a Rust struct with the values your shader needs (colours, textures, numbers) plus the shader file that uses them. Each entity points to a material asset; Bevy packs the values for the GPU.

## A 2D material

`assets/shaders/pulse.wgsl`:

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> time: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pulse = 0.5 + 0.5 * sin(time * 3.0);
    return vec4<f32>(color.rgb * pulse, color.a);
}
```

The Rust side:

```rust
use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct PulseMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[uniform(1)]
    time: f32,
}

impl Material2d for PulseMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/pulse.wgsl".into()
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Material2dPlugin::<PulseMaterial>::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, tick)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PulseMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(80.0))),
        MeshMaterial2d(materials.add(PulseMaterial { color: LinearRgba::new(0.2, 0.8, 1.0, 1.0), time: 0.0 })),
    ));
}

fn tick(time: Res<Time>, mut materials: ResMut<Assets<PulseMaterial>>) {
    for (_, material) in materials.iter_mut() {
        material.time = time.elapsed_secs();
    }
}
```

## 3D materials

The same pattern with `Material` and `MeshMaterial3d`. To reuse Bevy's lighting and only change a few things, **extend** `StandardMaterial`:

```rust
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct Glow {
    #[uniform(100)]
    intensity: f32,
}

impl MaterialExtension for Glow {
    fn fragment_shader() -> ShaderRef {
        "shaders/glow.wgsl".into()
    }
}

type GlowMaterial = ExtendedMaterial<StandardMaterial, Glow>;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<GlowMaterial>::default()))
        .run();
}
```

(Use binding indices 100 and up so they do not collide with `StandardMaterial`.)

## WESL and imports

Bevy 0.20 builds shaders with WESL: `import` other files and share code. See the `shaders` examples in the Bevy repo.

## Where to look

- Bevy's `examples/shader` folder: one example per technique (storage buffers, compute, post-process, instancing...).
- The [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) tutorial for the GPU side.
- For particles, [[bevy_hanabi]] saves a lot of work.

## Next

[[Windows]].
