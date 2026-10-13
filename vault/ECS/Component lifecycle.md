---
icon: sprout
tags: [ecs, advanced]
---
# Component lifecycle

What happens when a component is added, changed or removed, and how to hook in. #ecs #advanced

## Required components

`#[require(...)]` makes Bevy add other components automatically, using `Default` or a function:

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Fuel(f32);

fn full_tank() -> Fuel { Fuel(100.0) }

#[derive(Component)]
#[require(Velocity, Transform, Fuel = full_tank())]
struct Rocket;

fn spawn(mut commands: Commands) {
    commands.spawn(Rocket);   // also gets Velocity, Transform and Fuel(100)
}
```

An entity can still override: `commands.spawn((Rocket, Fuel(10.0)))`. Requirements are transitive, so `Sprite` pulls in `Transform`, `Visibility` and more.

## Lifecycle events

Bevy triggers an event at each moment in a component's life. Observe them like any [[Events and observers|event]]:

| Event | When |
|:--|:--|
| `Add<T>` | the entity gets `T` for the first time |
| `Insert<T>` | `T` is inserted (also when it replaces an old one) |
| `Discard<T>` | `T` is about to be replaced or removed |
| `Remove<T>` | `T` is removed (or the entity is despawned) |
| `Despawn<T>` | the entity holding `T` is despawned |

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Enemy;

#[derive(Resource, Default)]
struct EnemyCount(u32);

fn count_up(_: On<Add<Enemy>>, mut count: ResMut<EnemyCount>) {
    count.0 += 1;
}

fn count_down(_: On<Remove<Enemy>>, mut count: ResMut<EnemyCount>) {
    count.0 -= 1;
}

fn main() {
    App::new()
        .init_resource::<EnemyCount>()
        .add_observer(count_up)
        .add_observer(count_down)
        .run();
}
```

Keeps the counter correct no matter how enemies come and go: spawn, despawn, `remove::<Enemy>()`, an entire subtree.

## Hooks

A component can run code on its own lifecycle, enforced for every entity:

```rust
use bevy::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;

#[derive(Component)]
#[component(on_add = on_add_light)]
struct Torch;

fn on_add_light(mut world: DeferredWorld, context: HookContext) {
    world.commands().entity(context.entity).insert(PointLight::default());
}
```

Prefer required components and observers; hooks are for invariants a component must always keep. Each component type can have at most one hook per event.

## Immutable components

`#[component(immutable)]` forbids `&mut T` access. The only way to change it is to insert a new value, which means hooks and observers always see every change. [[Relationships and hierarchy|Relationships]] use this.

## Removal detection

Use `Remove<T>` observers, or `RemovedComponents<T>` in a system ([[Change detection]]).

## Next

[[Reflection]].
