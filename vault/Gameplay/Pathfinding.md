---
icon: route
tags: [gameplay, ai, intermediate]
---
# Pathfinding

Walking straight at a target works until there is a wall. **Pathfinding** finds a route around obstacles. The classic answer is **A\*** (said "A star"), and on a grid it fits in forty lines. #ai #gameplay

## The idea

Think of the map as a graph: each free cell is a node, linked to its free neighbours. A* explores outward from the start, always expanding the cell that looks *cheapest overall*:

> **f = g + h**, where **g** is the real cost to get to a cell, and **h** is a *guess* of what is left (here, the straight grid distance to the goal).

A good guess makes it fast (it heads for the goal). A guess that is never too high makes the result **shortest**. The Manhattan distance, `|dx| + |dy|`, is that guess on a 4-direction grid.

## The map

A grid is a size and a set of walls. `open` says whether a cell can be walked:

```rust example=pathfinding
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

use bevy::prelude::*;

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
```

`IVec2` is a vector of integers, perfect for cells. `cmpge(..).all()` means "both coordinates are at least 0".

## A\*

```rust example=pathfinding
fn astar(grid: &Grid, start: IVec2, goal: IVec2) -> Option<Vec<IVec2>> {
    let guess = |cell: IVec2| (goal - cell).abs().element_sum();      // h
    let mut open = BinaryHeap::new();                                 // cheapest f first
    open.push(Reverse((guess(start), start.x, start.y)));
    let mut cost = HashMap::from([(start, 0)]);                       // g
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
```

Read it top to bottom:

1. `open` is a **priority queue** (`BinaryHeap`). `Reverse` turns Rust's max-heap into a min-heap, so `pop` gives the lowest `f`. The cell is stored as a tuple of numbers because `BinaryHeap` needs `Ord`.
2. Take the best cell. If it is the goal, walk `came_from` backwards to rebuild the path.
3. Otherwise look at each neighbour. If it is open and we found a **cheaper way** to it than any before, record it and queue it.
4. If the queue empties, there is no path: return `None`.

The path comes back **goal first**, so a walker can `pop()` the next cell off the end.

## Walking the path

The agent keeps its remaining cells in a component. A click turns into a goal and a path; a system follows it:

```rust example=pathfinding
#[derive(Component)]
struct Agent;

/// The cells still to walk, next one last.
#[derive(Component, Default)]
struct Path(Vec<IVec2>);

const CELL: f32 = 16.0;

fn cell_to_world(cell: IVec2) -> Vec2 {
    (cell.as_vec2() + 0.5) * CELL
}

fn world_to_cell(at: Vec2) -> IVec2 {
    (at / CELL).floor().as_ivec2()
}

fn set_goal(
    click: On<PointerClick>,
    grid: Res<Grid>,
    mut agent: Single<(&Transform, &mut Path), With<Agent>>,
) {
    let Some(at) = click.hit.position else { return };
    let (transform, path) = &mut *agent;
    let start = world_to_cell(transform.translation.truncate());
    if let Some(found) = astar(&grid, start, world_to_cell(at.truncate())) {
        path.0 = found;
    }
}

fn follow_path(time: Res<Time>, mut agents: Query<(&mut Transform, &mut Path)>) {
    for (mut transform, mut path) in &mut agents {
        let Some(&next) = path.0.last() else { continue };
        let delta = cell_to_world(next) - transform.translation.truncate();
        let step = (80.0 * time.delta_secs()).min(delta.length());
        transform.translation += (delta.normalize_or_zero() * step).extend(0.0);
        if delta.length() < 0.5 {
            path.0.pop();                   // reached it: on to the next
        }
    }
}
```

Click anywhere in the preview. The yellow dots are the cells left on the path. Click inside a wall and nothing happens: `astar` returns `None`.

> [!NOTE]
> The preview's world is centred on `(0, 0)`, so it shifts cells by half the map. The idea is the same: **convert between world and cell** in two small functions and keep them together.

```rust example=pathfinding
fn plugin(app: &mut App) {
    app.add_observer(set_goal)
        .add_systems(Update, (follow_path, show_path));
}

// Dots on the path, to see it.
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
            Sprite::from_color(Color::srgb(0.98, 0.75, 0.3), Vec2::splat(4.0)),
            Transform::from_translation(cell_to_world(cell).extend(0.5)),
        ));
    }
}

#[derive(Component)]
struct Dot;
```

## Make it better

| Want | Do |
|:--|:--|
| diagonal moves | add the 4 diagonals as neighbours; cost `1.4`, use the *octile* distance as the guess |
| terrain (mud is slow) | cost per cell instead of `+ 1`; keep the guess optimistic |
| smoother walking | after A*, drop cells that a straight line can skip (*string pulling*) |
| many agents, one goal | one **flow field**: a single sweep from the goal gives every cell its direction |
| a changing map | recompute when a wall changes, not every frame; cache by goal |
| a big map | **hierarchical** A*: plan over regions, then inside them |
| think less often | run it on a timer, or on a background task ([[Custom assets]] shows the task pool idiom) |

## Beyond grids: navmeshes

Free-form levels (3D, or 2D without a grid) use a **navigation mesh**: walkable polygons, with the path through polygons. You rarely write one; crates do it, and the agent just asks "path to here".

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| `pathfinding` 4.16 | A*, Dijkstra, BFS and more on any graph; no Bevy dependency | works today |
| `bevy_northstar` 0.7 | hierarchical grid pathfinding | ⏳ on 0.19 |
| `vleue_navigator` 0.16 | navmeshes, 2D and 3D | ⏳ on 0.19 |
| `bevy_landmass` 0.12 | navmesh agents, avoidance | ⏳ on 0.19 |

`pathfinding` is a good first library: replace the hand-written `astar` with `pathfinding::directed::astar::astar` and keep everything else.

## Next

Put it to work in [[Game AI]]. Grids come from [[Tilemaps]]; random maps from [[Procedural generation]].
