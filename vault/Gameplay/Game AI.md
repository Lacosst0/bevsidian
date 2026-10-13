---
icon: bot
tags: [gameplay, ai, intermediate]
---
# Game AI

Game AI is not machine learning. It is **rules that pick what a character does next**, written so it looks smart enough. Most of it is three ideas: *sense* the world, *decide* with a small model, *act* by changing components. #ai #gameplay

> [!NOTE]
> Bevy has no AI built in, and does not need it: a decision is a system that reads components and writes components. This note builds a guard from scratch. The ecosystem crates are at the end.

## The guard

Meet the guard (green). It walks between two posts, **chases** you when you come close (red), and **walks back** when you get away (yellow). Click the preview to move the blue square.

First, the data. The guard is an entity with a `State` and a `Route`:

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

Why `SEE` is smaller than `LOSE`: the gap is a **hysteresis**. If both were one number, a player standing right on the line would flip the guard between states every frame.

## A state machine

A **finite state machine** (FSM) has a list of states, one current state, and rules for moving between them. In ECS the current state is just a component, and "what the guard does" is one system per state, each skipping guards that are in another state.

The rules live in one system that does nothing but change the state:

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

Read it as a table: *in this state, if this is true, go to that state*. `set_if_neq` writes only on a real change, so `Changed<State>` is a clean trigger for anything that should happen **once** when a state is entered (a sound, an animation, a colour).

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

The preview above runs the rules and the colour only, so the guard changes colour but never moves. Add the behaviours.

## One system per behaviour

Each state's behaviour is a small system that filters on the state:

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

The `Without<Guard>` on the player query matters: Bevy refuses two queries that could borrow the same `Transform` at once, one mutable. `With<Player>` + `Without<Guard>` proves they never overlap.

## All together

Register everything, and click to send the player running:

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

Run it, then walk the blue square close to the guard, and far away. Run the systems in order (`chain`): the decision first, then the behaviour, so a guard reacts the same frame.

> [!TIP]
> Cleaner at scale: instead of an enum and `if state == ...` in every system, give each state **its own marker component** (`Patrolling`, `Chasing`) and let systems query `With<Chasing>`. Changing state is `insert` one and `remove` the other. The crate `seldom_state` does this for you.

## Sensing

Everything above senses with one number: distance. Real senses are a few lines each.

**Field of view.** The guard sees only in front. Compare its facing direction with the direction to the target using a dot product:

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

**Line of sight.** A wall must hide the player. Cast a ray from the guard to the player; if it hits a wall first, the player is hidden. With the physics engine this is one call ([[bevy_rapier]] or [[Avian]] both have ray casts); for meshes there is Bevy's `MeshRayCast`.

**Hearing and memory.** Store what the guard last saw, so it can search rather than forget at once:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct LastSeen { at: Vec2, age: Timer }
```

## Steering

Movement is a decision too. **Steering behaviours** return a *desired velocity*; you add them up. Each is a one-line function:

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

A zombie is `seek + separate`; a sheep is `seek(flock centre) + separate + flee(dog)`. Weighting and summing gives surprisingly lifelike crowds from very little code.

## Paths

`seek` walks in a straight line. With walls in the way you need a **path**: [[Pathfinding]].

## Utility AI

A state machine has hard rules. **Utility AI** gives every possible action a *score* from 0 to 1 and picks the best. Characters feel less robotic because they weigh things:

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

Then a system `choose`s on a `Timer` (think four times a second, not 60) and inserts the chosen action as a component.

## Behaviour trees

A **behaviour tree** is a tree of tasks: *sequence* (do these in order, stop if one fails), *selector* (try these until one works), and *leaf* tasks (`MoveTo`, `Attack`). They scale better than a state machine when behaviours nest ("attack" contains "approach, aim, fire"). Libraries provide them; you rarely write one by hand.

## Making it feel good

- **Think less often.** Decisions at 4 Hz are indistinguishable from 60 Hz. Use a repeating `Timer` ([[Time and timers]]) per brain, with a random offset so not every enemy thinks on the same frame.
- **React with delay.** A guard that notices you *instantly* feels unfair. A 0.3 s "!" first feels fair.
- **Telegraph.** Colour, sound, an icon: show the state ([[Gizmos]] are great for debug vision cones).
- **Add noise.** A random wiggle on the route or on timing breaks lockstep movement ([[Randomness]]).
- **Debug it.** Draw each state, the target and the cone. Bugs in AI are almost always a wrong state or a missing transition.

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

Status checked 2026-10-13 against crates.io.

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| `seldom_state` 0.16 | state machines as components | ⏳ on 0.18 |
| `bevior_tree` 0.11 | behaviour trees | ⏳ on 0.19 |
| `bevy_behave` 0.5 | behaviour trees, dynamic spawning | ⏳ on 0.18 |
| `big-brain` 0.22 | utility AI | ⏳ on 0.15, unmaintained |
| `bevy_landmass` 0.12 | navmesh agents and steering | ⏳ on 0.19 |
| `vleue_navigator` 0.16 | navmeshes | ⏳ on 0.19 |
| `bevy_northstar` 0.7 | hierarchical grid pathfinding | ⏳ on 0.19 |

None has a Bevy 0.20 release yet. Everything in this note is plain Bevy and works today, and a hand-written state machine is often the right size.

## Next

[[Pathfinding]] to get around walls, [[Randomness]] for varied behaviour, [[Physics]] for ray casts.
