---
icon: git-branch
tags: [ecs]
---
# Relationships and hierarchy

Entities can point at other entities. Bevy keeps both ends in sync for you. The famous example is the parent/child tree. #ecs

## Parents and children

A child has a [`ChildOf`] component naming its parent. The parent automatically gets a [`Children`] list.

```rust example=hierarchy fps=2
use bevy::prelude::*;

#[derive(Component)]
struct Ship;

fn spawn_ship(mut commands: Commands) {
    commands.spawn((
        Name::new("Ship"),
        Ship,
        children![Name::new("Captain"), Name::new("Cook")],
    ));
}

fn add_sailor(mut commands: Commands, ship: Single<Entity, With<Ship>>) {
    // Insert ChildOf: the Ship's Children list updates itself.
    commands.spawn((Name::new("Sailor"), ChildOf(*ship)));
}

fn sink(mut commands: Commands, ship: Single<Entity, With<Ship>>) {
    // despawn removes the entity *and everything below it*.
    commands.entity(*ship).despawn();
}
```

Look at the cards: the ship has `Children`, each sailor has `ChildOf`. Press **Add a sailor** (what `add_sailor` does), then **Sink the ship**.

Despawning a parent despawns the whole subtree. No crew is left behind.

## Ways to build a tree

```rust
use bevy::prelude::*;

fn build(mut commands: Commands) {
    // 1. children! macro, as above
    commands.spawn((Name::new("A"), children![Name::new("A1"), Name::new("A2")]));

    // 2. with_children closure
    commands.spawn(Name::new("B")).with_children(|parent| {
        parent.spawn(Name::new("B1"));
    });

    // 3. add a child later
    let c = commands.spawn(Name::new("C")).id();
    let c1 = commands.spawn(Name::new("C1")).id();
    commands.entity(c).add_child(c1);
}
```

## Walking the tree

```rust
use bevy::prelude::*;

fn walk(
    roots: Query<Entity, Without<ChildOf>>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
) {
    for root in &roots {
        // everything below a root:
        for descendant in children.iter_descendants(root) {
            if let Ok(name) = names.get(descendant) {
                info!("{name} is below {root}");
            }
        }
    }

    // everything above an entity:
    // for ancestor in parents.iter_ancestors(entity) { ... }
}
```

## Transforms follow the tree

A child's [`Transform`] is **relative to its parent**. Move the ship and the crew come along. Bevy computes the world position in [`GlobalTransform`]. Details in [[Transforms]].

## Your own relationships

`ChildOf`/`Children` are just one relationship. Derive your own, and Bevy keeps both sides consistent:

```rust
use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = TargetedBy)]
struct Targeting(Entity);

#[derive(Component)]
#[relationship_target(relationship = Targeting)]
struct TargetedBy(Vec<Entity>);

fn aim(mut commands: Commands) {
    let target = commands.spawn(Name::new("Dummy")).id();
    commands.spawn((Name::new("Archer"), Targeting(target)));
    commands.spawn((Name::new("Mage"), Targeting(target)));
    // `target` now has TargetedBy([archer, mage]) without you writing it.
}

fn report(query: Query<(&Name, &TargetedBy)>) {
    for (name, hunters) in &query {
        info!("{name} is targeted by {} entities", hunters.len());
    }
}
```

Relationship components are *immutable*: to retarget, insert a new `Targeting(other)`. Use them for "owner of", "attached to", "member of", "inventory slot in"...

## Next

[[Advanced queries]]: querying through relationships, and more.
