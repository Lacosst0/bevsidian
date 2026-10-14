---
parent: Queries
icon: eye
tags: [ecs]
---
# Change detection

Bevy remembers *when* every component and resource was last written. Ask for "only what changed since I last ran" and skip the rest. #ecs

## Changed and Added

```rust example=change fps=2
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);

fn spawn_team(mut commands: Commands) {
    commands.spawn((Name::new("Ann"), Health(10)));
    commands.spawn((Name::new("Bob"), Health(7)));
}

fn regenerate(mut query: Query<(&Name, &mut Health)>) {
    for (name, mut health) in &mut query {
        if health.0 < 10 && name.as_str() == "Ann" {
            health.0 += 1;
        }
    }
}

fn report_changes(query: Query<(&Name, &Health), Changed<Health>>, mut log: ResMut<Log>) {
    for (name, health) in &query {
        log.say(format!("{name}'s health changed to {}", health.0));
    }
}

fn report_new(query: Query<&Name, Added<Health>>, mut log: ResMut<Log>) {
    for name in &query {
        log.say(format!("{name} got a Health component"));
    }
}
```

Step through a few frames. Everybody is "new" and "changed" on the first frame, then the log goes quiet: nothing changes. Press **Hit Bob**: only Bob's line appears on the next report.

| Filter | Matches entities whose component... |
|:--|:--|
| `Added<T>` | was added since this system last ran |
| `Changed<T>` | was added **or** written since this system last ran |

## What counts as a change?

Getting `&mut T` is not a change; **writing through it** is. Bevy marks the component as changed when you `DerefMut` it, even if the value stays the same.

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq)]
struct Health(u32);

fn clamp(mut query: Query<&mut Health>) {
    for mut health in &mut query {
        // Writes `health` even when nothing changes => marks it changed.
        health.0 = health.0.min(100);

        // Only writes when there is something to write:
        if health.0 > 100 {
            health.0 = 100;
        }

        // Or compare first, and set only if different:
        health.set_if_neq(Health(100));
    }
}
```

If something expensive listens to `Changed<Health>`, avoid false positives with `set_if_neq` or an `if`.

## Resources

`Res<T>` and `ResMut<T>` have `is_changed()` and `is_added()`:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct Score(u32);

fn announce(score: Res<Score>) {
    if score.is_changed() && !score.is_added() {
        info!("score is now {}", score.0);
    }
}
```

or use the run condition `resource_changed::<Score>`.

## Per-item checks

Inside a loop you can ask `Ref<T>` for the change flags without filtering:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Health(u32);

fn log(query: Query<Ref<Health>>) {
    for health in &query {
        if health.is_changed() {
            info!("changed: {}", health.0);
        }
    }
}
```

## Removed components

A removed component is gone, so there is nothing to query. Use the `Remove<T>` observer ([[Component lifecycle]]) or the `RemovedComponents<T>` parameter:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Burning;

fn extinguished(mut removed: RemovedComponents<Burning>) {
    for entity in removed.read() {
        info!("{entity} stopped burning");
    }
}
```

## Next

[[Relationships and hierarchy]].
