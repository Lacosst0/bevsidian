---
icon: box
tags: [ecs]
---
# Entities and components

An **entity** is an id. A **component** is a piece of data you hang on it. Behaviour comes from which components an entity has. #ecs

## Components

Any struct or enum can be a component. Derive [`Component`]:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);                 // one value

#[derive(Component)]
struct Stats { strength: u8, speed: u8 }   // named fields

#[derive(Component)]
enum Mood { Calm, Angry }           // states that exclude each other

#[derive(Component)]
struct Player;                      // a marker: no data, just a label
```

Markers are how you say "this is the player" so a [[Queries|query]] can pick it out.

## Spawning

An entity gets its components when it is spawned. Group them in a tuple:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Health(u32);
#[derive(Component)] struct Player;

fn spawn(mut commands: Commands) {
    commands.spawn((Name::new("Hero"), Player, Health(100)));
    commands.spawn(Name::new("Nobody"));      // one component is fine
    let empty: Entity = commands.spawn_empty().id();   // keep the id for later
}
```

[`Name`] is a built-in component that gives entities a readable label in logs and inspectors.

An entity can hold **one component of each type**. Inserting a second `Health` replaces the first.

## Adding and removing later

```rust
use bevy::prelude::*;

#[derive(Component)] struct Poisoned;

fn poison(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(Poisoned);     // add (or replace)
    commands.entity(entity).remove::<Poisoned>(); // take away
    commands.entity(entity).despawn();            // delete the whole entity
}
```

Adding a component at runtime is a cheap way to change behaviour: systems that look for `Poisoned` start (or stop) seeing the entity. [[Commands]] has a button-driven demo.

## Bundles and required components

A tuple of components is a **bundle**. Spawn the same bundle again and again by wrapping it in a function:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Health(u32);
#[derive(Component)] struct Enemy;

fn goblin() -> impl Bundle {
    (Name::new("Goblin"), Enemy, Health(12))
}

fn spawn(mut commands: Commands) {
    commands.spawn(goblin());
    commands.spawn(goblin());
}
```

Some components need others to work. A **required component** is added for you with a default:

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
#[require(Velocity, Transform)]   // spawning Ship also adds both
struct Ship;
```

Bevy uses this everywhere: spawning [`Camera2d`] also adds the camera, projection and transform it needs. [[Component lifecycle]] has more.

## Design tips

- **Small components.** `Health`, `Speed`, `Team` instead of one big `Character`. Systems then ask only for what they use, and run in parallel.
- **Data, not behaviour.** A component should hold values. Put the logic in [[Systems]].
- **Markers over booleans.** Prefer a `Frozen` marker to `is_frozen: bool`: queries can filter on it.
- **Enums for exclusive states**, markers for states that can overlap.

## Where things live

All entities live in a **World**, which is owned by the [[App and plugins|App]]. Systems never touch the world directly; they declare what they need through parameters like `Query` and `Res`, and Bevy hands it over.

## Next

[[Systems]] to act on this data, [[Queries]] to find it.
