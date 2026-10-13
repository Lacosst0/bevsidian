---
icon: filter
tags: [ecs, beginner]
---
# Queries

A [`Query`] asks the world: *give me every entity that has these components*. It is how almost every system gets its data. #ecs #beginner

## A first query

`Query<(&Health, &Poison)>` means "entities that have **both** `Health` and `Poison`; I'll read them". Add `mut` for write access. The loop runs once per match.

```rust example=queries fps=2
use bevy::prelude::*;

#[derive(Component)]
struct Hero;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct Poison(u32);

fn spawn_party(mut commands: Commands) {
    commands.spawn((Name::new("Hero"), Hero, Health(30)));
    commands.spawn((Name::new("Goblin"), Enemy, Health(12)));
    commands.spawn((Name::new("Wolf"), Enemy, Health(18), Poison(2)));
}

fn poison_damage(mut query: Query<(&mut Health, &Poison)>) {
    for (mut health, poison) in &mut query {
        health.0 = health.0.saturating_sub(poison.0);
    }
}
```

Step forward. Only the wolf loses health, because only the wolf has `Poison`. Press **Poison the goblin** and watch the next step: the goblin starts to lose health too. Nothing in the system changed; the *data* did.

## Filters

A filter narrows the match without fetching the component:

| Query | Matches entities... |
|:--|:--|
| `Query<&mut Health, With<Hero>>` | that also have `Hero` |
| `Query<&mut Health, Without<Poison>>` | that do not have `Poison` |
| `Query<&Health, (With<Enemy>, Without<Poison>)>` | both rules (a tuple means *and*) |
| `Query<&Health, Or<(With<Hero>, With<Poison>)>>` | either rule |
| `Query<&Health, Changed<Health>>` | whose `Health` changed ([[Change detection]]) |
| `Query<&Health, Added<Health>>` | that just got `Health` |

```rust example=queries fps=2
fn rest(mut query: Query<&mut Health, Without<Poison>>) {
    for mut health in &mut query {
        health.0 = (health.0 + 1).min(30);
    }
}
```

Anyone *without* poison recovers a point per frame. Poison the goblin: it stops resting.

## Optional components and names

`Option<&T>` matches whether or not the component is there:

```rust example=queries fps=2
fn report(query: Query<(&Name, Option<&Poison>)>, mut log: ResMut<Log>) {
    for (name, poison) in &query {
        match poison {
            Some(poison) => log.say(format!("{name} is poisoned ({})", poison.0)),
            None => log.say(format!("{name} is fine")),
        }
    }
}
```

(`Log` is the preview's stand-in for `info!`.)

## Getting entities and single items

```rust
use bevy::prelude::*;

#[derive(Component)] struct Health(u32);
#[derive(Component)] struct Player;

fn examples(
    everyone: Query<(Entity, &Health)>,          // Entity gives you the id
    player: Query<&Health, With<Player>>,
    mut commands: Commands,
) {
    for (entity, health) in &everyone {
        if health.0 == 0 {
            commands.entity(entity).despawn();
        }
    }

    // One specific entity: `get`. Exactly one match: `single`.
    if let Ok(health) = player.single() {
        info!("player has {}", health.0);
    }
}
```

`single()` returns an error when there are zero or several matches, so you handle it instead of crashing. [[Advanced queries]] shows `Single<...>`, which skips the system silently instead.

## Read or write?

Use `&T` unless you must change it. Read-only queries let more systems run in parallel, and `&mut` marks the component as [[Change detection|changed]] only when you actually write through it.

## Next

[[Commands]] to spawn and despawn, [[Advanced queries]] when the basics are not enough.
