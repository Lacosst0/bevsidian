---
icon: terminal
tags: [ecs, beginner]
---
# Commands

Systems run in parallel, so they cannot add and remove entities on the spot. [`Commands`] **queues** the change; Bevy applies the queue between systems. #ecs #beginner

## The four everyday commands

Each button runs the matching command on the preview's world. Press them in any order.

```rust example=commands
use bevy::prelude::*;

#[derive(Component)]
struct Slime;

#[derive(Component)]
struct Shield(u32);

fn spawn_slime(mut commands: Commands) {
    // spawn: a new entity with these components
    commands.spawn((Name::new("Slime"), Slime));
}

fn demo(mut commands: Commands, slime: Single<Entity, With<Slime>>) {
    // insert: add (or replace) a component on an existing entity
    commands.entity(*slime).insert(Shield(5));

    // remove: take a component away
    commands.entity(*slime).remove::<Shield>();

    // despawn: delete the entity and all its components
    commands.entity(*slime).despawn();
}
```

Look at the entity numbers on the cards: after `despawn`, spawning again reuses the slot, but it is a different entity.

## Why deferred?

```rust
use bevy::prelude::*;

#[derive(Component)] struct Coin;

fn collect(mut commands: Commands, coins: Query<Entity, With<Coin>>) {
    for coin in &coins {
        commands.entity(coin).despawn();   // still in the query until the system ends!
    }
}
```

The despawn happens after `collect` finishes. While the system runs, the query still sees the coin. This is what makes it safe to loop over a query and delete as you go.

## Spawn and keep the id

```rust
use bevy::prelude::*;

#[derive(Component)] struct Weapon;
#[derive(Component)] struct Owner(Entity);

fn equip(mut commands: Commands) {
    let hero = commands.spawn(Name::new("Hero")).id();
    commands.spawn((Weapon, Owner(hero)));
}
```

`.id()` hands you the `Entity` right away, even though the entity does not exist yet. You can use it in other commands in the same system.

## Spawn with children

```rust
use bevy::prelude::*;

fn tree(mut commands: Commands) {
    commands.spawn((
        Name::new("Tree"),
        children![Name::new("Trunk"), Name::new("Leaves")],
    ));
}
```

More in [[Relationships and hierarchy]].

## Custom commands and closures

Run any code with world access later:

```rust
use bevy::prelude::*;

fn reset(mut commands: Commands) {
    commands.queue(|world: &mut World| {
        world.clear_entities();
    });
}
```

Triggering an [[Events and observers|event]] is a command as well: `commands.trigger(MyEvent)`.

## Do it yourself right now

If you have `&mut World` ([[Systems#Exclusive systems|exclusive systems]], tests, startup in `main`), skip `Commands`: `world.spawn(...)` and `world.entity_mut(e).insert(...)` apply immediately.

## Next

[[Resources]] for global data, then [[Events and observers]].
