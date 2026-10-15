//! `Pathfinding.md`: an agent that finds its way around walls with A*. Click to send it.

// Map sizes are tiny constants: the casts cannot wrap.
#![allow(clippy::cast_possible_wrap)]

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

use bevy::prelude::*;

use super::stage::{self, Sky, spawn_sky};
use crate::example::{Example, Setup, observer, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky), startup!(spawn_world)],
    functions: &[
        observer!(set_goal),
        update!(follow_path),
        update!(show_path),
    ],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

fn setup(_: &World) -> Setup {
    Box::new(|_| {})
}

const CELL: f32 = 16.0;
const MAP: [&str; 10] = [
    "....................",
    ".......#............",
    ".......#....######..",
    ".......#..........#.",
    "..####.#..######..#.",
    ".......#..#.......#.",
    ".......####.#######.",
    "...........#........",
    "..########.#.######.",
    "....................",
];

#[derive(Resource)]
struct Grid {
    size: IVec2,
    walls: HashSet<IVec2>,
}

impl Grid {
    fn open(&self, cell: IVec2) -> bool {
        cell.cmpge(IVec2::ZERO).all() && cell.cmplt(self.size).all() && !self.walls.contains(&cell)
    }
}

#[derive(Component)]
struct Agent;

/// The cells still to walk, next one last.
#[derive(Component, Default)]
struct Path(Vec<IVec2>);

#[derive(Component)]
struct Dot;

fn cell_to_world(cell: IVec2) -> Vec2 {
    (cell.as_vec2() + 0.5) * CELL - stage::SIZE / 2.0
}

fn world_to_cell(at: Vec2) -> IVec2 {
    ((at + stage::SIZE / 2.0) / CELL).floor().as_ivec2()
}

fn spawn_world(mut commands: Commands) {
    let mut walls = HashSet::new();
    for (row, line) in MAP.iter().enumerate() {
        for (column, tile) in line.chars().enumerate() {
            if tile == '#' {
                let cell = IVec2::new(column as i32, (MAP.len() - 1 - row) as i32);
                walls.insert(cell);
                commands.spawn((
                    Sprite::from_color(Color::srgb(0.26, 0.28, 0.36), Vec2::splat(CELL - 1.0)),
                    Transform::from_translation(cell_to_world(cell).extend(0.0)),
                ));
            }
        }
    }
    commands.insert_resource(Grid {
        size: IVec2::new(MAP[0].len() as i32, MAP.len() as i32),
        walls,
    });
    commands.spawn((
        Agent,
        Path::default(),
        Sprite::from_color(stage::colour(2), Vec2::splat(12.0)),
        Transform::from_translation(cell_to_world(IVec2::new(0, 9)).extend(1.0)),
    ));
}

fn astar(grid: &Grid, start: IVec2, goal: IVec2) -> Option<Vec<IVec2>> {
    let guess = |cell: IVec2| (goal - cell).abs().element_sum();
    let mut open = BinaryHeap::new();
    open.push(Reverse((guess(start), start.x, start.y)));
    let mut cost = HashMap::from([(start, 0)]);
    let mut came_from = HashMap::new();
    while let Some(Reverse((_, x, y))) = open.pop() {
        let cell = IVec2::new(x, y);
        if cell == goal {
            let mut path = vec![cell];
            while let Some(&before) = came_from.get(path.last()?) {
                path.push(before);
            }
            path.pop(); // the start
            return Some(path);
        }
        for step in [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y] {
            let next = cell + step;
            let new_cost = cost[&cell] + 1;
            if grid.open(next) && cost.get(&next).is_none_or(|&old| new_cost < old) {
                cost.insert(next, new_cost);
                came_from.insert(next, cell);
                open.push(Reverse((new_cost + guess(next), next.x, next.y)));
            }
        }
    }
    None
}

fn set_goal(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    grid: Res<Grid>,
    mut agent: Single<(&Transform, &mut Path), With<Agent>>,
) {
    let Some(at) = click.hit.position else { return };
    if !sky.contains(click.entity) {
        return;
    }
    let (transform, path) = &mut *agent;
    let start = world_to_cell(transform.translation.truncate());
    if let Some(found) = astar(&grid, start, world_to_cell(at.truncate())) {
        path.0 = found;
    }
}

fn follow_path(time: Res<Time>, mut agents: Query<(&mut Transform, &mut Path)>) {
    for (mut transform, mut path) in &mut agents {
        let Some(&next) = path.0.last() else { continue };
        let to = cell_to_world(next);
        let delta = to - transform.translation.truncate();
        let step = (80.0 * time.delta_secs()).min(delta.length());
        transform.translation += (delta.normalize_or_zero() * step).extend(0.0);
        if delta.length() < 0.5 {
            path.0.pop();
        }
    }
}

fn show_path(
    agent: Single<&Path, (With<Agent>, Changed<Path>)>,
    dots: Query<Entity, With<Dot>>,
    mut commands: Commands,
) {
    for dot in &dots {
        commands.entity(dot).despawn();
    }
    for &cell in &agent.0 {
        commands.spawn((
            Dot,
            Sprite::from_color(stage::colour(1), Vec2::splat(4.0)),
            Transform::from_translation(cell_to_world(cell).extend(0.5)),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_way_around_the_walls() {
        let walls = MAP
            .iter()
            .enumerate()
            .flat_map(|(row, line)| {
                line.chars()
                    .enumerate()
                    .filter(|&(_, tile)| tile == '#')
                    .map(move |(column, _)| IVec2::new(column as i32, (MAP.len() - 1 - row) as i32))
            })
            .collect();
        let grid = Grid {
            size: IVec2::new(20, 10),
            walls,
        };
        let path = astar(&grid, IVec2::new(0, 9), IVec2::new(19, 0)).expect("a path");
        assert_eq!(path[0], IVec2::new(19, 0), "the goal comes first");
        assert!(path.iter().all(|&c| grid.open(c)));
        assert!(
            astar(&grid, IVec2::new(0, 9), IVec2::new(7, 8)).is_none(),
            "inside a wall"
        );
    }
}
