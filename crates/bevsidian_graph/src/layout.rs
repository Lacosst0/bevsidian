//! Where notes belong in the graph, before the simulation polishes it.
//!
//! - [`tree`]: the vault grown like a tree. The hub is the trunk in the middle; its branches
//!   (the areas) leave it in every direction, each branch splits into the notes under it, and
//!   every generation sits further out than the one before. A branch with more notes below it
//!   gets a wider wedge of the circle.
//! - [`line`]: a reading path in a straight line, its core steps in order and its optional
//!   and extra steps hanging above and below the step they branch from.

use std::f32::consts::TAU;

use bevsidian_core::vault::StepKind;
use bevy::math::Vec2;

/// Distance between consecutive core steps of a path.
pub const STEP_GAP: f32 = 190.0;
/// Distance between the rows of steps hanging off a path.
pub const BRANCH_GAP: f32 = 150.0;

/// Room between one generation and the next.
const RING_GAP: f32 = 110.0;
/// Distance between the staggered rows of one generation, when its notes do not fit side by
/// side.
const ROW_GAP: f32 = 64.0;
/// Most staggered rows a generation of one branch may need before the ring moves outwards.
const MAX_ROWS: usize = 4;

/// Home positions for the notes of a tree: `parent[n]` is the note `n` grows from (`None`
/// for the root, and for notes that are not part of the tree), `space[n]` the room it needs
/// (its diameter, with its label). The root sits at the origin.
pub fn tree(parent: &[Option<usize>], root: usize, space: &[f32]) -> Vec<Vec2> {
    let count = parent.len();
    let mut homes = vec![Vec2::ZERO; count];
    if count == 0 {
        return homes;
    }
    let mut children = vec![Vec::new(); count];
    for (child, &up) in parent.iter().enumerate() {
        if let Some(up) = up.filter(|_| child != root) {
            children[up].push(child);
        }
    }
    // Generations, from the root outwards.
    let mut generations = vec![vec![root]];
    loop {
        let next: Vec<usize> = generations
            .last()
            .into_iter()
            .flatten()
            .flat_map(|&n| children[n].iter().copied())
            .collect();
        if next.is_empty() || generations.len() > count {
            break;
        }
        generations.push(next);
    }
    // Notes at the tips below each note: the more, the wider its wedge.
    let mut tips = vec![1.0_f32; count];
    for generation in generations.iter().rev() {
        for &n in generation {
            if !children[n].is_empty() {
                tips[n] = children[n].iter().map(|&c| tips[c]).sum();
            }
        }
    }
    // Each note's wedge of the circle: (start, width).
    let mut wedge = vec![(0.0_f32, TAU); count];
    for generation in &generations {
        for &n in generation {
            let (mut at, width) = wedge[n];
            for &c in &children[n] {
                let part = width * tips[c] / tips[n];
                wedge[c] = (at, part);
                at += part;
            }
        }
    }
    // Generations sit at evenly spaced radii. The spacing is the smallest at which every
    // generation fits (no branch needs more than MAX_ROWS rows) and the generations, with
    // their staggered rows, do not run into each other.
    let rows_at = |generation: &[usize], radius: f32| -> Vec<(usize, usize)> {
        let mut parents: Vec<usize> = generation.iter().filter_map(|&n| parent[n]).collect();
        parents.dedup();
        parents
            .into_iter()
            .map(|p| {
                let widest = children[p].iter().map(|&c| space[c]).fold(1.0, f32::max);
                let fits = ((wedge[p].1 * radius / widest).floor() as usize).max(1);
                (p, children[p].len().div_ceil(fits))
            })
            .collect()
    };
    let half = |rows: usize| rows.saturating_sub(1) as f32 * ROW_GAP / 2.0;
    let fits = |step: f32| {
        let deepest: Vec<usize> = generations
            .iter()
            .enumerate()
            .skip(1)
            .map(|(d, generation)| {
                rows_at(generation, d as f32 * step)
                    .iter()
                    .map(|&(_, r)| r)
                    .max()
                    .unwrap_or(1)
            })
            .collect();
        deepest.iter().all(|&r| r <= MAX_ROWS)
            && deepest
                .windows(2)
                .all(|w| half(w[0]) + half(w[1]) + RING_GAP <= step)
            && step >= space[root] / 2.0 + RING_GAP
    };
    let mut step = RING_GAP;
    while !fits(step) && step < 20_000.0 {
        step *= 1.05;
    }
    for (d, generation) in generations.iter().enumerate().skip(1) {
        let radius = d as f32 * step;
        for (p, count) in rows_at(generation, radius) {
            for (k, &c) in children[p].iter().enumerate() {
                let (start, width) = wedge[c];
                let angle = start + width / 2.0;
                let row = if count > 1 { k % count } else { 0 };
                let offset = (row as f32 - (count - 1) as f32 / 2.0) * ROW_GAP;
                homes[c] = Vec2::from_angle(angle) * (radius + offset);
            }
        }
    }
    homes
}

/// Positions for the steps of a path, in their order: core steps left to right on one line
/// centred on the origin; every other step below or above the core step before it,
/// alternating, further out the more there are.
pub fn line(kinds: &[StepKind]) -> Vec<Vec2> {
    let cores = kinds.iter().filter(|&&k| k == StepKind::Core).count();
    let first = -(cores.saturating_sub(1) as f32) * STEP_GAP / 2.0;
    let mut core = None::<usize>;
    let mut hanging: usize = 0;
    kinds
        .iter()
        .map(|&kind| {
            if kind == StepKind::Core {
                core = Some(core.map_or(0, |c| c + 1));
                hanging = 0;
                Vec2::new(first + core.unwrap_or(0) as f32 * STEP_GAP, 0.0)
            } else {
                hanging += 1;
                // Below, above, further below, …; before any core step, under the start.
                let row = hanging.div_ceil(2) as f32;
                let side = if hanging % 2 == 1 { -1.0 } else { 1.0 };
                let x = first + core.unwrap_or(0) as f32 * STEP_GAP;
                Vec2::new(x, side * row * BRANCH_GAP)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tree_grows_outwards_in_generations() {
        // 0 is the trunk; 1, 2, 3 its branches; 4, 5 grow from 1; 6 from 2; 7 from 4.
        let parent = [
            None,
            Some(0),
            Some(0),
            Some(0),
            Some(1),
            Some(1),
            Some(2),
            Some(4),
        ];
        let homes = tree(&parent, 0, &[80.0; 8]);
        assert_eq!(homes[0], Vec2::ZERO);
        for (child, up) in [(1, 0), (2, 0), (3, 0), (4, 1), (5, 1), (6, 2), (7, 4)] {
            assert!(
                homes[child].length() > homes[up].length(),
                "{child} is further out than {up}"
            );
        }
        // No two notes on the same spot.
        for a in 0..8 {
            for b in (a + 1)..8 {
                assert!(homes[a].distance(homes[b]) > 40.0, "{a} and {b}");
            }
        }
        // The bigger branch (1, with three notes below) has the wider wedge: its children
        // spread further apart than 2's single child is from anything.
        assert!(homes[4].distance(homes[5]) > 40.0);
    }

    #[test]
    fn many_notes_in_one_branch_stagger_into_rows() {
        let mut parent = vec![None, Some(0)];
        parent.extend((0..30).map(|_| Some(1)));
        let homes = tree(&parent, 0, &[80.0; 32]);
        for a in 0..32 {
            for b in (a + 1)..32 {
                assert!(homes[a].distance(homes[b]) > 30.0, "{a} and {b}");
            }
        }
    }

    #[test]
    fn paths_lie_on_a_line_with_detours_beside_it() {
        use StepKind::{Core, Extra, Optional};
        let at = line(&[Core, Optional, Extra, Core, Core]);
        assert_eq!(at[0], Vec2::new(-STEP_GAP, 0.0));
        assert_eq!(at[3], Vec2::new(0.0, 0.0));
        assert_eq!(at[4], Vec2::new(STEP_GAP, 0.0));
        assert_eq!(
            at[1],
            Vec2::new(-STEP_GAP, -BRANCH_GAP),
            "below its core step"
        );
        assert_eq!(at[2], Vec2::new(-STEP_GAP, BRANCH_GAP), "then above");
    }
}
