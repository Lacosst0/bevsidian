---
icon: search
tags: [ecs]
---
# Advanced queries

Everything beyond `Query<&T>`. You rarely need it all at once; skim and come back. #ecs

## Single

When there is exactly one match (the player, the camera), `Single` is clearer than looping:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Player;

fn follow(
    player: Single<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    camera.translation.x = player.translation.x;
}
```

If zero or several entities match, the system is **skipped** and Bevy logs a warning. To handle that yourself use `Query::single()`, or `Option<Single<...>>`.

> [!NOTE]
> The `Without<Player>` is needed: both queries would otherwise ask for `&Transform` mutably and immutably on possibly the same entity, and Bevy refuses that.

## Disjoint access

Two queries touching the same component conflict unless a filter proves they never overlap:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Friend;
#[derive(Component)] struct Foe;

fn separate(
    mut friends: Query<&mut Transform, With<Friend>>,
    mut foes: Query<&mut Transform, (With<Foe>, Without<Friend>)>,
) {}
```

When filters cannot prove it, use [[System params#ParamSet|ParamSet]].

## More filters and data

| You write | You get |
|:--|:--|
| `Has<T>` | a `bool`: does the entity have `T`? Never filters |
| `AnyOf<(&A, &B)>` | `(Option<&A>, Option<&B>)`, matches if at least one exists |
| `Option<&T>` | the component if present |
| `Ref<T>` | read-only with change flags |
| `Entity` | the id |
| `&Name` | readable label |
| `Allow<Disabled>` | also see [disabled entities](#disabling-entities) |

```rust
use bevy::prelude::*;

#[derive(Component)] struct Health(u32);
#[derive(Component)] struct Shield;

fn show(query: Query<(&Health, Has<Shield>)>) {
    for (health, shielded) in &query {
        info!("{} hp, shielded: {shielded}", health.0);
    }
}
```

## Get a specific entity

```rust
use bevy::prelude::*;

#[derive(Component)] struct Health(u32);
#[derive(Component)] struct Target(Entity);

fn hurt(mut healths: Query<&mut Health>, attackers: Query<&Target>) {
    for target in &attackers {
        if let Ok(mut health) = healths.get_mut(target.0) {
            health.0 = health.0.saturating_sub(1);
        }
    }
    // Several at once (they must be different entities):
    // let [a, b] = healths.get_many_mut([e1, e2]).unwrap();
}
```

## Pairs of entities

```rust
use bevy::prelude::*;

#[derive(Component)] struct Radius(f32);

fn collide(mut query: Query<(&Transform, &Radius)>) {
    // Every unordered pair, once.
    let mut pairs = query.iter_combinations_mut();
    while let Some([(a, ra), (b, rb)]) = pairs.fetch_next() {
        if a.translation.distance(b.translation) < ra.0 + rb.0 {
            info!("touching");
        }
    }
}
```

## Parallel iteration

For many entities with heavy work per item:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Velocity(Vec2);

fn integrate(mut query: Query<(&mut Transform, &Velocity)>) {
    query.par_iter_mut().for_each(|(mut transform, velocity)| {
        transform.translation += velocity.0.extend(0.0);
    });
}
```

Don't reach for this first: Bevy already runs *different systems* in parallel.

## Your own query data

Bundle a repeated query into a named type:

```rust
use bevy::prelude::*;
use bevy::ecs::query::QueryData;

#[derive(Component)] struct Health(u32);
#[derive(Component)] struct Armor(u32);

#[derive(QueryData)]
#[query_data(mutable)]
struct Fighter {
    entity: Entity,
    health: &'static mut Health,
    armor: Option<&'static Armor>,
}

fn fight(mut fighters: Query<Fighter>) {
    for mut fighter in &mut fighters {
        let armor = fighter.armor.map_or(0, |a| a.0);
        fighter.health.0 = fighter.health.0.saturating_sub(5u32.saturating_sub(armor));
    }
}
```

## Disabling entities

Add the `Disabled` component and an entity disappears from every ordinary query (rendering, physics, everything), without despawning. Ask for it explicitly with `Allow<Disabled>`. Good for object pools and pausing a subtree.

## Dynamic and one-off queries

With `&mut World` you can query on the spot:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Enemy;

fn count(world: &mut World) -> usize {
    world.query_filtered::<Entity, With<Enemy>>().iter(world).count()
}
```

## Next

[[System params]]: more system parameters beyond `Query`.
