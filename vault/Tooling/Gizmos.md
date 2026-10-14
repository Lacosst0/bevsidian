---
icon: compass
tags: [tooling]
---
# Gizmos

Gizmos draw temporary lines and shapes for one frame. They are the quickest way to **see** invisible things: velocities, ranges, paths, colliders. #tooling

## Draw in a system

Ask for [`Gizmos`] and draw. Everything lasts one frame, so draw every frame:

```rust
use bevy::prelude::*;

fn debug_draw(mut gizmos: Gizmos, player: Single<&Transform, With<Camera2d>>) {
    gizmos.line_2d(Vec2::ZERO, Vec2::new(100.0, 50.0), Color::srgb(1.0, 0.3, 0.3));
    gizmos.circle_2d(Isometry2d::from_translation(player.translation.truncate()), 40.0, Color::srgb(0.3, 1.0, 0.4));
    gizmos
        .grid_2d(Isometry2d::IDENTITY, UVec2::new(16, 9), Vec2::splat(80.0), LinearRgba::gray(0.1))
        .outer_edges();
}
```

3D versions end in `_3d` and take positions as `Vec3`:

```rust
use bevy::prelude::*;

fn debug_3d(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::Y * 2.0, Color::WHITE);
    gizmos.sphere(Isometry3d::IDENTITY, 1.0, Color::srgb(0.3, 0.6, 1.0));
    gizmos.cube(Transform::from_xyz(3.0, 0.5, 0.0), Color::srgb(1.0, 0.8, 0.2));
    gizmos.arrow(Vec3::ZERO, Vec3::new(2.0, 1.0, 0.0), Color::srgb(0.2, 1.0, 0.4));
}
```

## Show a velocity or range

```rust
use bevy::prelude::*;

#[derive(Component)] struct Velocity(Vec2);
#[derive(Component)] struct Enemy { sight: f32 }

fn show_velocity(mut gizmos: Gizmos, query: Query<(&Transform, &Velocity)>) {
    for (transform, velocity) in &query {
        let from = transform.translation.truncate();
        gizmos.arrow_2d(from, from + velocity.0, Color::srgb(1.0, 1.0, 0.0));
    }
}

fn show_sight(mut gizmos: Gizmos, query: Query<(&Transform, &Enemy)>) {
    for (transform, enemy) in &query {
        gizmos.circle_2d(Isometry2d::from_translation(transform.translation.truncate()), enemy.sight, Color::srgba(1.0, 0.2, 0.2, 0.5));
    }
}
```

Run them only in debug builds: `.add_systems(Update, show_velocity.run_if(|| cfg!(debug_assertions)))`.

## Style

Change thickness or turn on dashed lines for all gizmos in `GizmoConfigStore`:

```rust
use bevy::{gizmos::config::GizmoConfigStore, prelude::*};

fn style(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 3.0;
}
```

Make your own group (`#[derive(Default, Reflect, GizmoConfigGroup)]`, `init_gizmo_group`) to switch categories on and off separately, e.g. "physics" and "ai".

## Gizmos on entities

Some components have built-in gizmos: `ShowAabbGizmo` on a mesh, light gizmos (`ShowLightGizmo`) for lights, `TransformGizmo`-style axes (`gizmos.axes(...)`). Text can be drawn with gizmos too in 0.20 (`text_gizmos` examples).

## Next

[[Diagnostics and logging]].
