---
icon: lightbulb
tags: [3d]
---
# Lighting

3D objects are black until something lights them. Bevy has three light types, plus ambient and environment light. #3d

## The three lights

```rust
use bevy::prelude::*;

fn lights(mut commands: Commands) {
    // A light bulb: shines in all directions.
    commands.spawn((
        PointLight { intensity: 1_000_000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));

    // A torch: a cone.
    commands.spawn((
        SpotLight { intensity: 2_000_000.0, outer_angle: 0.6, shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(-3.0, 4.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // The sun: parallel rays, no position, only a direction.
    commands.spawn((
        DirectionalLight { illuminance: 10_000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

| Light | Where it matters | Cost |
|:--|:--|:--|
| `DirectionalLight` | the sun; direction from its `Transform` | cheap, one per scene |
| `PointLight` | lamps, fire, explosions | medium, six shadow maps |
| `SpotLight` | flashlights, stage lights | medium |

Units are physical (lumens, lux). If a scene looks too dark or burnt, change `intensity` by 10×, not by 2×.

## Shadows

Off by default. Turn them on per light with `shadow_maps_enabled: true`. Then mark which meshes cast and receive:

- Every mesh casts and receives by default.
- Add `NotShadowCaster` or `NotShadowReceiver` to an entity to opt out.

For the sun over a big world, shadows are split into zones near to far:

```rust
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};

fn sun(mut commands: Commands) {
    commands.spawn((
        DirectionalLight { shadow_maps_enabled: true, ..default() },
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            maximum_distance: 80.0,
            ..default()
        }
        .build(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

## Ambient light: the fill

Light that comes from everywhere equally, so shadowed areas are not pitch black:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            ..default()
        })
        .run();
}
```

## Image-based lighting

A skybox picture can light the scene realistically (reflections too). Attach an `EnvironmentMapLight` to the camera:

```rust
use bevy::prelude::*;

fn camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/diffuse.ktx2"),
            specular_map: asset_server.load("environment_maps/specular.ktx2"),
            intensity: 1000.0,
            ..default()
        },
    ));
}
```

Light probes, baked lightmaps, SSAO, volumetric fog and ray-traced "Solari" lighting exist for bigger budgets: see Bevy's `3d` examples.

## Tips

- A **single sun plus ambient** looks good for most outdoor games.
- Emissive materials do not light other objects; add a point light next to them.
- Many point lights with shadows get slow. Use few, or turn shadows off on small ones.

## Next

[[Models and glTF]].
