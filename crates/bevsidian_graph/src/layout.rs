//! Where notes belong in the graph, before the simulation polishes it.
//!
//! - [`radial`]: the whole vault radiating from its hub. Notes sit on rings by link distance
//!   from the hub; each note's subtree gets a slice of the circle sized by how many notes it
//!   holds, so branches point outwards and stay together. A ring too crowded for its notes
//!   splits into staggered bands (neighbours alternate between them), so the graph grows
//!   with the square root of the notes rather than in proportion.
//! - [`line`]: a reading path in a straight line, its core steps in order and its optional
//!   and extra steps hanging above and below the step they branch from.

use std::{collections::VecDeque, f32::consts::TAU};

use bevsidian_core::vault::StepKind;
use bevy::math::Vec2;

/// Distance between rings (world units).
pub const RING_GAP: f32 = 230.0;
/// Distance between the bands a crowded ring splits into.
pub const BAND_GAP: f32 = 105.0;
/// Distance between consecutive core steps of a path.
pub const STEP_GAP: f32 = 190.0;
/// Distance between the rows of steps hanging off a path.
pub const BRANCH_GAP: f32 = 150.0;

/// Home positions for `count` nodes linked by `links` (undirected, by index), radiating from
/// `root` at the origin. `space[i]` is the room node `i` needs (its diameter, with its label).
/// Nodes the root cannot reach go on an outer ring.
pub fn radial(count: usize, links: &[(usize, usize)], root: usize, space: &[f32]) -> Vec<Vec2> {
    let mut neighbours = vec![Vec::new(); count];
    for &(a, b) in links {
        if a != b && a < count && b < count {
            neighbours[a].push(b);
            neighbours[b].push(a);
        }
    }
    for list in &mut neighbours {
        list.sort_unstable();
        list.dedup();
    }
    // Breadth-first from the root: depth and the parent each note hangs from.
    let mut depth = vec![usize::MAX; count];
    let mut children = vec![Vec::new(); count];
    let mut order = Vec::with_capacity(count);
    let mut queue = VecDeque::new();
    if root < count {
        depth[root] = 0;
        queue.push_back(root);
    }
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &next in &neighbours[node] {
            if depth[next] == usize::MAX {
                depth[next] = depth[node] + 1;
                children[node].push(next);
                queue.push_back(next);
            }
        }
    }
    // The unreachable hang from the root, one ring further out than anything else.
    let outer = order.iter().map(|&n| depth[n]).max().unwrap_or(0) + 1;
    if root < count {
        for (node, d) in depth.iter_mut().enumerate() {
            if *d == usize::MAX {
                *d = outer;
                children[root].push(node);
                order.push(node);
            }
        }
    }
    // Leaves under each note (deepest first), for the size of its slice.
    let mut weight = vec![1.0_f32; count];
    for &node in order.iter().rev() {
        let below: f32 = children[node].iter().map(|&c| weight[c]).sum();
        weight[node] = below.max(1.0);
    }
    // Slices of the circle, from the root outwards.
    let mut angle = vec![0.0_f32; count];
    let mut slice = vec![(0.0_f32, TAU); count];
    for &node in &order {
        let (start, width) = slice[node];
        angle[node] = start + width / 2.0;
        let total: f32 = children[node].iter().map(|&c| weight[c]).sum();
        let mut at = start;
        for &child in &children[node] {
            let share = width * weight[child] / total.max(1.0);
            slice[child] = (at, share);
            at += share;
        }
    }
    // Each ring starts a gap beyond the last; if its notes do not fit around it side by
    // side, it splits into bands further out, which take turns going round.
    let deepest = depth.iter().copied().max().unwrap_or(0);
    let mut radius = vec![0.0_f32; count];
    let mut outer = 0.0_f32;
    for d in 1..=deepest {
        let mut ring: Vec<usize> = (0..count).filter(|&n| depth[n] == d).collect();
        if ring.is_empty() {
            continue;
        }
        ring.sort_by(|&a, &b| angle[a].total_cmp(&angle[b]));
        let base = outer + RING_GAP;
        let needed: f32 = ring.iter().map(|&n| space[n]).sum();
        let capacity =
            |bands: usize| -> f32 { (0..bands).map(|b| TAU * (base + b as f32 * BAND_GAP)).sum() };
        let mut bands = 1;
        while capacity(bands) < needed && bands < ring.len() {
            bands += 1;
        }
        for (i, &n) in ring.iter().enumerate() {
            radius[n] = base + (i % bands) as f32 * BAND_GAP;
        }
        outer = base + (bands - 1) as f32 * BAND_GAP;
    }
    (0..count)
        .map(|n| Vec2::from_angle(angle[n]) * radius[n])
        .collect()
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
    fn rings_go_outwards_from_the_root() {
        // 0 is the root; 1 and 2 hang from it, 3 from 1; 4 is unreachable.
        let homes = radial(5, &[(0, 1), (0, 2), (1, 3)], 0, &[80.0; 5]);
        assert_eq!(homes[0], Vec2::ZERO);
        let r = |i: usize| homes[i].length();
        assert!((r(1) - r(2)).abs() < 0.01, "same ring");
        assert!(r(3) > r(1) + RING_GAP - 0.01, "a ring further out");
        assert!(r(4) > r(3), "the unreachable outside everything");
        // 3 lies in 1's slice: closer to 1 than to 2.
        assert!(homes[3].distance(homes[1]) < homes[3].distance(homes[2]));
    }

    #[test]
    fn crowded_rings_split_into_bands() {
        let links: Vec<_> = (1..60).map(|i| (0, i)).collect();
        let homes = radial(60, &links, 0, &[100.0; 60]);
        let mut radii: Vec<f32> = homes[1..].iter().map(|h| h.length().round()).collect();
        radii.sort_by(f32::total_cmp);
        radii.dedup();
        assert!(radii.len() > 1, "more than one band: {radii:?}");
        let around: f32 = radii.iter().map(|r| r * TAU).sum();
        assert!(around >= 59.0 * 100.0, "room for every note");
        assert!(
            radii[0] == RING_GAP,
            "the first band where a lone ring would be"
        );
        // Neighbours by angle take turns between bands.
        assert!((homes[1].length() - homes[2].length()).abs() > 1.0);
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
