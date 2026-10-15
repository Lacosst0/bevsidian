---
icon: bot
tags: [gameplay, ai]
---
# Game AI

Game AI isn't machine learning. It's **rules that pick what a character does next**, just smart enough to look smart: *sense* the world, *decide*, *act* by changing components. Bevy needs nothing special for it; a decision is a system that reads components and writes others. #ai #gameplay

## The guard

Meet the guard. It walks between two posts (green), **chases** you when you come close (red) and **walks back** when you get away (yellow). You're the ship: click to move it.

First the data. The guard is an entity with a `State` and a `Route`:

```rust example=ai
use bevy::prelude::*;

const SEE: f32 = 70.0;   // sees the player closer than this
const LOSE: f32 = 110.0; // loses them farther than this

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Guard;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum State {
    Patrol,
    Chase,
    Return,
}

/// The two ends of the guard's beat, and which one it walks to.
#[derive(Component)]
struct Route {
    points: [Vec2; 2],
    next: usize,
}

/// Moves `from` towards `to`, never past it.
fn step_towards(from: &mut Transform, to: Vec2, speed: f32, time: &Time) {
    let delta = to - from.translation.truncate();
    let step = (speed * time.delta_secs()).min(delta.length());
    from.translation += (delta.normalize_or_zero() * step).extend(0.0);
}
```

`SEE` is smaller than `LOSE` on purpose. With one number, a player standing on the line would flip the guard back and forth every frame. The gap (**hysteresis**) stops that.

## A state machine

A **state machine** is a set of states, one current state and rules for moving between them. In ECS the state is a component. One system does nothing but apply the rules:

```rust example=ai
fn watch_for_player(
    player: Single<&Transform, (With<Player>, Without<Guard>)>,
    mut guards: Query<(&Transform, &Route, &mut State), With<Guard>>,
) {
    for (guard, route, mut state) in &mut guards {
        let distance = guard.translation.distance(player.translation);
        let next = match *state {
            State::Patrol | State::Return if distance < SEE => State::Chase,
            State::Chase if distance > LOSE => State::Return,
            State::Return
                if guard.translation.truncate().distance(route.points[route.next]) < 4.0 =>
            {
                State::Patrol
            }
            other => other,
        };
        state.set_if_neq(next);
    }
}
```

Read it as a table: *in this state, if this holds, go there*. `set_if_neq` writes only on a real change, so `Changed<State>` fires **once** per new state: the place for a sound, an animation or a colour.

```rust example=ai
fn show_state(mut guards: Query<(&State, &mut Sprite), Changed<State>>) {
    for (state, mut sprite) in &mut guards {
        sprite.color = match state {
            State::Patrol => Color::srgb(0.55, 0.85, 0.45),
            State::Chase => Color::srgb(0.95, 0.45, 0.4),
            State::Return => Color::srgb(0.98, 0.75, 0.3),
        };
    }
}
```

So far the guard changes colour but never moves. Time for behaviours.

## One system per behaviour

Each state gets a small system that skips guards in any other state:

```rust example=ai
fn patrol(time: Res<Time>, mut guards: Query<(&mut Transform, &mut Route, &State)>) {
    for (mut transform, mut route, state) in &mut guards {
        if *state != State::Patrol {
            continue;
        }
        step_towards(&mut transform, route.points[route.next], 40.0, &time);
        if transform.translation.truncate().distance(route.points[route.next]) < 1.0 {
            route.next = 1 - route.next;   // turn around
        }
    }
}
```

```rust example=ai
fn chase(
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<Guard>)>,
    mut guards: Query<(&mut Transform, &State), With<Guard>>,
) {
    for (mut transform, state) in &mut guards {
        if *state == State::Chase {
            step_towards(&mut transform, player.translation.truncate(), 70.0, &time);
        }
    }
}

fn go_back(time: Res<Time>, mut guards: Query<(&mut Transform, &Route, &State)>) {
    for (mut transform, route, state) in &mut guards {
        if *state == State::Return {
            step_towards(&mut transform, route.points[route.next], 50.0, &time);
        }
    }
}
```

`Without<Guard>` on the player query matters: two queries that could both reach one `Transform`, one of them mutably, are refused. The filter proves they never overlap ([[Advanced queries]]).

## All together

Register it all, and click to send the ship running:

```rust example=ai
fn move_player(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    if let (true, Some(at)) = (sky.contains(click.entity), click.hit.position) {
        commands.entity(*player).insert(Target(at.truncate()));
    }
}

fn plugin(app: &mut App) {
    app.add_observer(move_player).add_systems(
        Update,
        (watch_for_player, (patrol, chase, go_back, show_state)).chain(),
    );
}

#[derive(Component)]
struct Sky;

#[derive(Component)]
struct Target(Vec2);
```

Fly close to the guard, then away. `chain` runs the decision before the behaviours, so the guard reacts in the same frame.

> [!TIP]
> At scale, swap the enum for **one marker per state** (`Patrolling`, `Chasing`) and query `With<Chasing>`. Changing state is an `insert` and a `remove`. `seldom_state` does exactly this.

## Sensing

Distance is the simplest sense. The others are a few lines each.

**Field of view**: the guard only sees in front. Compare the way it faces with the way to the target:

```rust
use bevy::prelude::*;

/// Is `target` inside the cone `fov` radians wide, `range` long, in front of `from`?
fn sees(from: &Transform, target: Vec2, range: f32, fov: f32) -> bool {
    let to = target - from.translation.truncate();
    if to.length() > range {
        return false;
    }
    let facing = (from.rotation * Vec3::X).truncate();
    facing.angle_to(to).abs() <= fov / 2.0
}
```

**Line of sight**: cast a ray from guard to player; if a wall comes first, the player is hidden. Physics engines do it in one call ([[bevy_rapier]], [[Avian]]); for meshes there's Bevy's `MeshRayCast`.

**Memory**: remember where the player was last seen, so the guard searches instead of forgetting at once:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct LastSeen { at: Vec2, age: Timer }
```

## Steering

**Steering behaviours** each return a desired velocity; add them up to move. Each is a line or two:

```rust
use bevy::prelude::*;

fn seek(position: Vec2, target: Vec2, speed: f32) -> Vec2 {
    (target - position).normalize_or_zero() * speed
}

fn flee(position: Vec2, threat: Vec2, speed: f32) -> Vec2 {
    -seek(position, threat, speed)
}

/// Seek, but slow down inside `radius` so it stops *on* the target.
fn arrive(position: Vec2, target: Vec2, speed: f32, radius: f32) -> Vec2 {
    let distance = position.distance(target);
    seek(position, target, speed * (distance / radius).min(1.0))
}

/// Push away from neighbours that are too close (stops crowds stacking).
fn separate(position: Vec2, neighbours: &[Vec2], radius: f32, speed: f32) -> Vec2 {
    neighbours
        .iter()
        .filter(|n| position.distance(**n) < radius)
        .map(|n| flee(position, *n, speed))
        .sum()
}
```

A zombie is `seek + separate`; a sheep is `seek(flock) + separate + flee(dog)`. Weighted sums make lifelike crowds from very little code. With walls in the way, `seek` isn't enough: you need a path ([[Pathfinding]]).

## Utility AI

**Utility AI** scores every action from 0 to 1 and picks the best. Characters weigh their options instead of following hard rules:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Needs { hunger: f32, fear: f32, tired: f32 }   // each 0..1

#[derive(Debug, PartialEq, Clone, Copy)]
enum Action { Eat, Flee, Sleep, Wander }

fn choose(needs: &Needs) -> Action {
    let options = [
        (Action::Flee, needs.fear * 1.5),        // fear counts more
        (Action::Eat, needs.hunger),
        (Action::Sleep, needs.tired * 0.8),
        (Action::Wander, 0.1),                   // the fallback
    ];
    options
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(action, _)| action)
        .unwrap()
}
```

Call `choose` on a timer (four times a second is plenty) and insert the result as a component.

## Behaviour trees

A **behaviour tree** nests tasks: a *sequence* runs children in order until one fails, a *selector* tries them until one works, *leaves* do the work (`MoveTo`, `Attack`). Reach for one when behaviours nest deeply ("attack" = approach, aim, fire); use a crate rather than writing your own.

## Making it feel good

- **Think less often.** 4 Hz looks the same as 60. Give each brain a repeating `Timer` with a random offset ([[Time and timers]]).
- **React with a delay.** Instant noticing feels unfair; a 0.3 s "!" feels fair.
- **Telegraph.** Show the state with colour, sound or an icon.
- **Add noise** to routes and timing so enemies don't move in lockstep ([[Randomness]]).
- **Draw it.** Most AI bugs are a wrong state or a missing transition; [[Gizmos]] make them obvious.

```rust
use bevy::prelude::*;

fn draw_vision(mut gizmos: Gizmos, guards: Query<&Transform, With<Guard>>) {
    for guard in &guards {
        gizmos.circle_2d(
            Isometry2d::from_translation(guard.translation.truncate()),
            70.0,
            Color::srgb(1.0, 0.4, 0.4),
        );
    }
}

#[derive(Component)]
struct Guard;
```

## Crates

| Crate | For | Bevy |
|:--|:--|:--|
| `seldom_state` 0.16 | state machines as components | #0.19 |
| `bevior_tree` 0.11 | behaviour trees | #0.19 |
| `bevy_behave` 0.5 | behaviour trees, dynamic spawning | #0.19 |
| `big-brain` 0.22 | utility AI | #0.19 |
| `bevy_landmass` 0.12 | navmesh agents and steering | #0.19 |
| `vleue_navigator` 0.16 | navmeshes | #0.19 |
| `bevy_northstar` 0.7 | hierarchical grid pathfinding | #0.19 |

None supports #0.20 yet. Everything above is plain Bevy, and a hand-written state machine is often all you need.

## Next

[[Pathfinding]] to get around walls, [[Randomness]] for varied behaviour, [[Physics]] for ray casts.
