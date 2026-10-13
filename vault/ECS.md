---
icon: boxes
---
# ECS

Bevy is built on an **Entity Component System** (ECS). Data lives in *components*, which are attached to *entities*; logic lives in *systems*. #ecs #beginner

## Entities and components

An entity is just an ID. You give it meaning by inserting components. The preview above the code runs it in a real Bevy world and shows the entity it spawns:

```rust preview=spawn-player
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, Health(50), Transform::default()));
}
```

> [!NOTE]
> Since Bevy 0.20, **resources are components** stored on singleton entities, so most of the APIs below work for both.

## Systems

Systems are plain Rust functions whose parameters describe the data they need. Bevy runs systems in parallel when their data access does not conflict.

Step through it one system at a time (Bevy's built-in *stepping*), or press play: `spawn_player` runs once at startup, then `regenerate` heals the player every frame.

```rust preview=regenerate
fn regenerate(time: Res<Time>, mut query: Query<&mut Health, With<Player>>) {
    for mut health in &mut query {
        health.0 = (health.0 + 1).min(100);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, regenerate)
        .run();
}
```

## Queries

| Query | Meaning |
|:--|:--|
| `Query<&T>` | read `T` on every entity that has it |
| `Query<&mut T>` | write `T` |
| `Query<&T, With<U>>` | only entities that also have `U` |
| `Query<Entity, Changed<T>>` | entities whose `T` changed since the system last ran |

## Observers and events

*Observers* react to events immediately instead of polling every frame. They are also how [[Scenes and BSN|scenes]] attach behaviour to entities with `on(...)`. Here a system triggers `Damage` each frame, and the observer runs right away:

```rust preview=damage
#[derive(EntityEvent)]
struct Damage {
    entity: Entity,
    amount: u32,
}

fn on_damage(damage: On<Damage>, mut health: Query<&mut Health>) {
    if let Ok(mut health) = health.get_mut(damage.entity) {
        health.0 = health.0.saturating_sub(damage.amount);
    }
}

fn attack(players: Query<Entity, With<Player>>, mut commands: Commands) {
    for player in &players {
        commands.trigger(Damage { entity: player, amount: 10 });
    }
}
```

## See also

- [[Scenes and BSN]] builds whole entity hierarchies at once.
- [[Rendering]] covers the components the renderer reads, such as `Mesh3d` and `Camera3d`.
