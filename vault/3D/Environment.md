---
icon: mountain
tags: [3d]
---
# Environment

Sky, fog, atmosphere: what makes a 3D scene feel like a place. #3d

## Background

By default the background is a flat colour:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.53, 0.75, 0.95)))   // day blue
        .run();
}
```

## Skybox

A cube-map image drawn behind everything, attached to the camera:

```rust
use bevy::{core_pipeline::Skybox, prelude::*};

fn camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Skybox {
            image: Some(asset_server.load("environment_maps/sky.ktx2")),
            brightness: 1000.0,
            ..default()
        },
    ));
}
```

Pair it with an `EnvironmentMapLight` using the same sky so lighting and reflections match ([[Lighting]]).

## Fog

Distance fog hides pop-in and adds depth:

```rust
use bevy::prelude::*;

fn camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        DistanceFog {
            color: Color::srgb(0.7, 0.8, 0.9),
            falloff: FogFalloff::Linear { start: 20.0, end: 120.0 },
            ..default()
        },
    ));
}
```

Fog falloff modes: `Linear`, `Exponential`, `ExponentialSquared`, `Atmospheric`. Volumetric fog (light shafts) is a separate `FogVolume` + `VolumetricFog` feature.

## Atmosphere

Bevy can render a physically based sky: the sun's position changes the colours from dawn to dusk. Add an `Atmosphere` to the camera and a `DirectionalLight`; animate the light's rotation to move the sun. See the `atmosphere` and `atmospheric_fog` examples.

## Water, terrain, trees

Not built in. Use [[bevy_water]], build a terrain mesh from noise ([[Meshes and materials]]), or import terrain as glTF ([[Models and glTF]]).

## Next

Time to move around: [[Input#Keyboard and mouse|Keyboard and mouse]], or the [[A 3D game]] path.
