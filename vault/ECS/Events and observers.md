---
icon: zap
tags: [ecs, beginner]
---
# Events and observers

An **event** says "this just happened". An **observer** is a system that runs *immediately* when a matching event is triggered. No polling, no waiting for the next frame. #ecs #beginner

## Trigger and observe

```rust example=events
use bevy::prelude::*;

#[derive(Component)]
struct Door;

/// Aimed at one entity.
#[derive(EntityEvent)]
struct Knock {
    entity: Entity,
}

fn spawn_house(mut commands: Commands) {
    commands.spawn((Name::new("Front door"), Door));
    commands.spawn((Name::new("Back door"), Door));
}

fn on_knock(knock: On<Knock>, names: Query<&Name>, mut log: ResMut<Log>) {
    if let Ok(name) = names.get(knock.entity) {
        log.say(format!("knock, knock: {name} heard it"));
    }
}

fn main() {
    App::new()
        .add_systems(Startup, spawn_house)
        .add_observer(on_knock)
        .run();
}
```

Press **Knock on the front door**. The observer's line appears at once. To fire from code: `commands.trigger(Knock { entity })`.

(`Log` is this page's stand-in for `info!`.)

## Two kinds of event

| Derive | Targets | Trigger |
|:--|:--|:--|
| `#[derive(Event)]` | nobody in particular ("global") | `commands.trigger(Ring { times: 2 })` |
| `#[derive(EntityEvent)]` | one entity (has an `entity` field) | `commands.trigger(Knock { entity })` |

```rust example=events
#[derive(Event)]
struct Ring {
    times: u32,
}

fn on_ring(ring: On<Ring>, mut log: ResMut<Log>) {
    log.say(format!("the bell rings {} times", ring.times));
}
```

An `On<E>` observer reads the event's fields straight through (`ring.times`).

## Watching one entity

`.observe(system)` attaches an observer to a single entity, so it only hears events aimed at *it*:

```rust
use bevy::prelude::*;

#[derive(EntityEvent)]
struct Open { entity: Entity }

fn setup(mut commands: Commands) {
    commands
        .spawn(Name::new("Chest"))
        .observe(|open: On<Open>, mut commands: Commands| {
            info!("{} opened", open.entity);
            commands.entity(open.entity).despawn();
        });
}
```

Closures work as observers too. This is how UI buttons get their click handlers in [[Scenes and BSN|BSN]]: `on(|_: On<PointerPress>| ...)`.

## Lifecycle events

Bevy triggers events when components come and go. Observe `Add<T>`, `Insert<T>`, `Remove<T>` and `Despawn<T>`:

```rust example=events
fn on_door_added(add: On<Add<Door>>, names: Query<&Name>, mut log: ResMut<Log>) {
    let name = names.get(add.entity).map_or("a door", Name::as_str);
    log.say(format!("{name} was built"));
}
```

Reset the preview and watch the log: both doors announce themselves. See [[Component lifecycle]].

## Events or messages?

| | Event + observer | [[Messages]] |
|:--|:--|:--|
| Runs | immediately, at the trigger | later, when a system reads them |
| Handler | an observer system | any system with a `MessageReader` |
| Good for | reactions: "a door opened" | streams: "all keys pressed this frame" |
| Can target an entity | yes | no |

If you are unsure, start with an observer.

## Order and surprises

Observers run in the middle of whatever triggered them, so they can see half-finished state. Keep them short; for heavy work trigger a smaller event or queue a [[Commands|command]].

## Next

[[Messages]], then [[Change detection]].
