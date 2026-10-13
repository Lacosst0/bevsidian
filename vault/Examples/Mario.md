---
icon: gamepad-2
tags: [example]
check: skip
---
# Mario

A one-screen platformer in a few systems and observers. Every preview on this page runs the same game, in a Bevy world of its own; each code block adds what it shows. Press play, or click the game to start it. #ecs #beginner

## The level

The sky, the ground and Mario are entities with a [`Sprite`] and a [`Transform`]. Gravity pulls every body with a `Velocity` down; colliding, walking and stomping are in the game's other systems.

```rust example=mario
#[derive(Component)]
struct Mario;

/// The background: clicks that hit nothing else land on it.
#[derive(Component)]
struct Sky;

#[derive(Component, Default)]
struct Velocity(Vec2);

fn spawn_level(sprites: Res<Sprites>, mut commands: Commands) {
    commands.spawn((
        Sky,
        Sprite::from_color(SKY, SIZE),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    let ground = Vec2::new(SIZE.x, 32.0);
    commands.spawn((
        Solid,
        Collider(ground),
        Sprite {
            custom_size: Some(ground),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            },
            ..Sprite::from_image(sprites.ground.clone())
        },
        Transform::from_xyz(0.0, -SIZE.y / 2.0 + 16.0, 0.0),
    ));
    commands.spawn((
        Mario,
        Velocity::default(),
        Collider(Vec2::new(12.0, 16.0)),
        Sprite::from_image(sprites.mario.clone()),
        Transform::from_translation(MARIO_START),
    ));
}

fn gravity(time: Res<Time>, mut bodies: Query<(&mut Transform, &mut Velocity), Without<Held>>) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut bodies {
        velocity.0.y = (velocity.0.y - GRAVITY * dt).max(-MAX_FALL);
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}
```

## Goombas

A repeating [`Timer`] in a resource spawns a goomba every two seconds, up to three at a time. They walk until they hit a wall or a platform, then turn around.

```rust example=mario
#[derive(Resource)]
struct GoombaTimer(Timer);

fn spawn_goombas(
    time: Res<Time>,
    mut timer: ResMut<GoombaTimer>,
    goombas: Query<(), With<Goomba>>,
    sprites: Res<Sprites>,
    mut commands: Commands,
) {
    if !timer.0.tick(time.delta()).just_finished() || goombas.iter().count() >= 3 {
        return;
    }
    commands.spawn((
        Goomba,
        Velocity(Vec2::new(-GOOMBA_SPEED, 0.0)),
        Collider(Vec2::splat(16.0)),
        Sprite::from_image(sprites.goomba.clone()),
        Transform::from_xyz(130.0, 90.0, 1.0),
    ));
}
```

## Platforms

Picking turns clicks into [`PointerClick`] events on the entity under the pointer, with the hit position in world space. Click the sky to place a platform there:

```rust example=mario
fn spawn_platform(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    sprites: Res<Sprites>,
    mut commands: Commands,
) {
    // Only clicks on the sky: not on Mario, a goomba or another platform.
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    // Snap to the level's 16-unit grid.
    let at = (at.truncate() / 16.0).floor() * 16.0 + 8.0;
    commands.spawn((
        Solid,
        Collider(PLATFORM),
        Sprite {
            custom_size: Some(PLATFORM),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: false,
                stretch_value: 1.0,
            },
            ..Sprite::from_image(sprites.brick.clone())
        },
        Transform::from_translation(at.extend(0.0)),
    ));
}
```

## Dragging Mario

Drags come as [`PointerDragStart`], [`PointerDrag`] and [`PointerDragEnd`]. While held, gravity leaves Mario alone; let go above a goomba to stomp it.

```rust example=mario
/// Being dragged: gravity leaves it alone.
#[derive(Component)]
struct Held;

fn grab(drag: On<PointerDragStart>, marios: Query<(), With<Mario>>, mut commands: Commands) {
    if marios.contains(drag.entity) {
        commands.entity(drag.entity).insert(Held);
    }
}

fn drag_mario(drag: On<PointerDrag>, mut marios: Query<&mut Transform, With<Mario>>) {
    if let Ok(mut transform) = marios.get_mut(drag.entity) {
        // Screen y points down, world y up.
        transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.0);
    }
}

fn release(
    drag: On<PointerDragEnd>,
    mut marios: Query<&mut Velocity, With<Mario>>,
    mut commands: Commands,
) {
    if let Ok(mut velocity) = marios.get_mut(drag.entity) {
        velocity.0 = Vec2::ZERO;
        commands.entity(drag.entity).remove::<Held>();
    }
}
```

## The whole game

```rust example=mario
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GoombaTimer>()
        .add_systems(Startup, spawn_level)
        .add_systems(
            Update,
            (gravity, land, walk, stomp, squash, fall_out, spawn_goombas),
        )
        .add_observer(spawn_platform)
        .add_observer(grab)
        .add_observer(drag_mario)
        .add_observer(release)
        .run();
}
```

See [[ECS]] for components, systems and observers.
