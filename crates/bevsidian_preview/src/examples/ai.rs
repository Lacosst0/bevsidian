//! `Game AI.md`: a guard with three states. It patrols, chases the player it sees, and walks
//! back when it loses them. Click to move the player.

use bevy::prelude::*;

use super::stage::{self, Sky, spawn_sky};
use crate::example::{Example, Setup, observer, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky), startup!(spawn_actors), update!(walk)],
    functions: &[
        update!(watch_for_player),
        update!(patrol),
        update!(chase),
        update!(go_back),
        update!(show_state),
        observer!(move_player),
    ],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|_| {})
}

const SEE: f32 = 70.0;
const LOSE: f32 = 110.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Guard;

/// Where the player is walking.
#[derive(Component)]
struct Target(Vec2);

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

fn spawn_actors(mut commands: Commands) {
    let points = [Vec2::new(-110.0, -40.0), Vec2::new(110.0, 40.0)];
    for point in points {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.3, 0.32, 0.4), Vec2::splat(6.0)),
            Transform::from_translation(point.extend(0.0)),
        ));
    }
    commands.spawn((
        Player,
        Sprite::from_color(stage::colour(3), Vec2::splat(12.0)),
        Transform::from_xyz(100.0, -50.0, 1.0),
    ));
    commands.spawn((
        Guard,
        State::Patrol,
        Route { points, next: 1 },
        Sprite::from_color(stage::colour(2), Vec2::splat(16.0)),
        Transform::from_translation(points[0].extend(1.0)),
    ));
}

/// Moves `from` towards `to`, never past it.
fn step_towards(from: &mut Transform, to: Vec2, speed: f32, time: &Time) {
    let delta = to - from.translation.truncate();
    let step = (speed * time.delta_secs()).min(delta.length());
    from.translation += (delta.normalize_or_zero() * step).extend(0.0);
}

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

fn walk(time: Res<Time>, mut players: Query<(&mut Transform, &Target), With<Player>>) {
    for (mut transform, target) in &mut players {
        step_towards(&mut transform, target.0, 90.0, &time);
    }
}

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
                if guard
                    .translation
                    .truncate()
                    .distance(route.points[route.next])
                    < 4.0 =>
            {
                State::Patrol
            }
            other => other,
        };
        state.set_if_neq(next);
    }
}

fn patrol(time: Res<Time>, mut guards: Query<(&mut Transform, &mut Route, &State)>) {
    for (mut transform, mut route, state) in &mut guards {
        if *state != State::Patrol {
            continue;
        }
        step_towards(&mut transform, route.points[route.next], 40.0, &time);
        if transform
            .translation
            .truncate()
            .distance(route.points[route.next])
            < 1.0
        {
            route.next = 1 - route.next;
        }
    }
}

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

fn show_state(mut guards: Query<(&State, &mut Sprite), Changed<State>>) {
    for (state, mut sprite) in &mut guards {
        sprite.color = match state {
            State::Patrol => stage::colour(2),
            State::Chase => stage::colour(0),
            State::Return => stage::colour(1),
        };
    }
}
