---
icon: palette
---
# Rendering

Bevy renders with `wgpu`, which targets Vulkan, Metal, DirectX 12, WebGL2 and WebGPU. Rendering is driven by ordinary [[ECS]] components. #rendering

## Cameras

Nothing is drawn until a camera exists. Spawn `Camera2d` for 2D games and `Camera3d` for 3D:

```rust
fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

## Meshes and materials

A visible 3D object is an entity with a `Mesh3d` and a `MeshMaterial3d`:

```rust
fn spawn_cube(
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

> [!WARNING]
> On the web, WebGL2 has a few limitations, such as fewer storage buffers. Enable the `webgpu` feature for browsers that support WebGPU.

## UI

Bevy UI is made of `Node` entities laid out with flexbox and CSS grid. Text uses `Text` with nested `TextSpan`s, and this very page is rendered that way. #ui

- `Node`: size, padding, flex and grid layout
- `Text` / `TextSpan`: styled text runs
- `ImageNode`: images
- `Button` + observers: interaction

## Shaders

Custom materials are written in WGSL:

```wgsl
@fragment
fn fragment() -> @location(0) vec4<f32> {
    return vec4<f32>(0.6, 0.4, 0.9, 1.0);
}
```

Related: [[Scenes and BSN]] for spawning whole UI trees, and [[Welcome]] for an overview.
