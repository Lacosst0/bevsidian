---
icon: boxes
tags: [ecs, beginner]
---
# ECS

Bevy is built on an **Entity Component System**. Once this clicks, the rest of Bevy is easy, because the engine itself works the same way. #ecs #beginner

| Letter | What | Is |
|:--|:--|:--|
| **E**ntity | a thing in your game | just an id |
| **C**omponent | data on a thing | a plain Rust struct or enum |
| **S**ystem | logic | a plain Rust function |

Think of a database: entities are rows, components are columns, a system is a query that runs every frame.

## The whole idea in one demo

An entity is spawned with three components, a system heals it each frame, an observer reacts when it is hit. Press play, step through with the arrows, or hit the player yourself.

```rust example=ecs fps=4
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, Health(50), Transform::default()));
}
```

```rust example=ecs fps=4
fn regenerate(time: Res<Time>, mut query: Query<&mut Health, With<Player>>) {
    for mut health in &mut query {
        health.0 = (health.0 + 1).min(100);
    }
}

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, regenerate)
        .add_observer(on_damage)
        .run();
}
```

> [!NOTE]
> Since Bevy 0.20, **resources are components** stored on singleton entities. `Res<T>` still works exactly as before.

## The concepts, in the order to learn them

The solid path is [[Understanding ECS]]. Here is the map.

**Data**
- [[Entities and components]]: what is stored, and how to design it.
- [[Resources]]: global data.
- [[Relationships and hierarchy]]: parents, children and your own links.

**Logic**
- [[Systems]]: functions Bevy calls for you.
- [[Queries]] and [[Advanced queries]]: finding the entities you need.
- [[Commands]]: changing the world safely.
- [[System params]]: `Local`, `ParamSet`, your own parameters, one-shot systems.

**Reacting**
- [[Events and observers]]: run code *when* something happens.
- [[Messages]]: queue things up for systems to read later.
- [[Change detection]]: only do work when data changed.
- [[Component lifecycle]]: hooks, required components.

**Order and flow**
- [[Schedules and sets]]: when systems run, and in which order.
- [[Run conditions]]: skip systems when they do not apply.
- [[States]]: menus, levels, pause.
- [[Time and timers]]: frames, fixed steps, cooldowns.

**Robustness**
- [[Error handling]], [[Reflection]] and [[Testing]].

## Next

[[Entities and components]]. Then try it all in [[Your first game]].
