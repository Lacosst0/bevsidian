//! Where notes belong in the graph, before the simulation polishes it.
//!
//! - [`clusters`]: the whole vault as clusters of related notes (a folder each) packed
//!   around the hub's cluster in the middle, biggest first, each near the clusters it links
//!   to. A cluster is a sunflower around its centre note (its index), the best-linked notes
//!   closest, and a note linking out of its cluster sits on the side facing where it links.
//! - [`line`]: a reading path in a straight line, its core steps in order and its optional
//!   and extra steps hanging above and below the step they branch from.

use std::f32::consts::TAU;

use bevsidian_core::vault::StepKind;
use bevy::math::Vec2;

/// Distance between neighbouring clusters, and between the middle cluster and the others.
pub const CLUSTER_GAP: f32 = 70.0;
/// Distance between a cluster's notes, as a share of the room they need: the simulation
/// pushes apart the few that end up too close.
const PACKING: f32 = 0.8;
/// Distance between consecutive core steps of a path.
pub const STEP_GAP: f32 = 190.0;
/// Distance between the rows of steps hanging off a path.
pub const BRANCH_GAP: f32 = 150.0;

/// Directions tried for each cluster.
const DIRECTIONS: usize = 96;
/// How much being near linked clusters counts against being near the middle.
const AFFINITY: f32 = 0.5;

/// The golden angle: consecutive sunflower seeds turn by it, so none line up.
const GOLDEN: f32 = 2.399_963;

/// The centre of each group: the member with most links inside it (the earliest on a tie).
pub fn centres(groups: &[usize], links: &[(usize, usize)]) -> Vec<usize> {
    let group_count = groups.iter().max().map_or(0, |g| g + 1);
    let mut inner = vec![0_usize; groups.len()];
    for &(a, b) in links {
        if a != b && groups[a] == groups[b] {
            inner[a] += 1;
            inner[b] += 1;
        }
    }
    let mut centres = vec![usize::MAX; group_count];
    for (node, &group) in groups.iter().enumerate() {
        let best = centres[group];
        if best == usize::MAX || inner[node] > inner[best] {
            centres[group] = node;
        }
    }
    centres
}

/// Home positions for the nodes in `groups` (a group index per node), linked by `links`
/// (undirected, by index). `root`'s group sits in the middle with `root` at the origin; every
/// other group is a cluster around its [`centres`], packed around the middle. `space[i]` is the
/// room node `i` needs (its diameter, with its label).
pub fn clusters(
    groups: &[usize],
    links: &[(usize, usize)],
    root: usize,
    space: &[f32],
) -> Vec<Vec2> {
    let count = groups.len();
    if count == 0 {
        return Vec::new();
    }
    let group_count = groups.iter().max().map_or(0, |g| g + 1);
    let middle = groups[root.min(count - 1)];
    let mut centre = centres(groups, links);
    centre[middle] = root.min(count - 1);
    let mut members = vec![Vec::new(); group_count];
    for (node, &group) in groups.iter().enumerate() {
        members[group].push(node);
    }
    // Links between groups, and inside them.
    let mut between = vec![vec![0_usize; group_count]; group_count];
    let mut inner = vec![0_usize; count];
    for &(a, b) in links {
        let (ga, gb) = (groups[a], groups[b]);
        if a == b {
            continue;
        }
        if ga == gb {
            inner[a] += 1;
            inner[b] += 1;
        } else {
            between[ga][gb] += 1;
            between[gb][ga] += 1;
        }
    }
    // Each cluster's sunflower: seed spacing and how far it reaches.
    let spacing: Vec<f32> = members
        .iter()
        .map(|nodes| {
            let needed: f32 = nodes.iter().map(|&n| space[n]).sum();
            PACKING * needed / nodes.len().max(1) as f32
        })
        .collect();
    let reach: Vec<f32> = (0..group_count)
        .map(|g| {
            // The outermost seat, and room for a note there.
            spacing[g] * (members[g].len().saturating_sub(1) as f32).sqrt()
                + spacing[g] / PACKING / 2.0
        })
        .collect();
    // Pack the clusters around the middle, biggest first: each as close to the middle as
    // it fits, near the clusters it links to.
    let mut order: Vec<usize> = (0..group_count)
        .filter(|&g| g != middle && !members[g].is_empty())
        .collect();
    order.sort_by_key(|&g| (std::cmp::Reverse(members[g].len()), g));
    let mut middle_at = vec![Vec2::ZERO; group_count];
    let mut placed = vec![middle];
    for g in order {
        let linked: f32 = placed.iter().map(|&p| between[g][p] as f32).sum();
        let best = (0..DIRECTIONS)
            .map(|i| {
                let direction = Vec2::from_angle(TAU * i as f32 / DIRECTIONS as f32);
                let at = direction * nearest_free(direction, reach[g], &placed, &middle_at, &reach);
                // Average distance to the clusters it links to, by how much it links.
                let apart: f32 = placed
                    .iter()
                    .map(|&p| between[g][p] as f32 * at.distance(middle_at[p]))
                    .sum::<f32>()
                    / linked.max(1.0);
                (at, at.length() + AFFINITY * apart)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or(Vec2::ZERO, |(at, _)| at);
        middle_at[g] = best;
        placed.push(g);
    }
    // Seats in each sunflower: the centre in the middle, the best-linked closest, and a note
    // linking out on the side facing where it links.
    let mut homes = vec![Vec2::ZERO; count];
    let mut neighbours = vec![Vec::new(); count];
    for &(a, b) in links {
        if a != b {
            neighbours[a].push(b);
            neighbours[b].push(a);
        }
    }
    for g in 0..group_count {
        let mut nodes = members[g].clone();
        if nodes.is_empty() {
            continue;
        }
        nodes.sort_by_key(|&n| (n != centre[g], std::cmp::Reverse(inner[n]), n));
        let seats: Vec<Vec2> = (0..nodes.len())
            .map(|k| Vec2::from_angle(k as f32 * GOLDEN) * spacing[g] * (k as f32).sqrt())
            .collect();
        let outer = seats.iter().map(|s| s.length()).fold(1.0, f32::max);
        let mut free = vec![true; seats.len()];
        for &node in &nodes {
            // Where its links outside the cluster go, and how many of its links do.
            let outside: Vec<Vec2> = neighbours[node]
                .iter()
                .filter(|&&n| groups[n] != g)
                .map(|&n| (middle_at[groups[n]] - middle_at[g]).normalize_or_zero())
                .collect();
            let pull = outside.iter().copied().sum::<Vec2>() / neighbours[node].len().max(1) as f32;
            let facing = pull.normalize_or_zero();
            let strength = pull.length();
            let seat = (0..seats.len())
                .filter(|&k| free[k])
                .min_by(|&a, &b| {
                    let cost = |k: usize| {
                        let seat: Vec2 = seats[k];
                        let away = 1.0 - seat.normalize_or_zero().dot(facing);
                        seat.length() / outer + away * strength
                    };
                    cost(a).total_cmp(&cost(b))
                })
                .unwrap_or(0);
            free[seat] = false;
            homes[node] = middle_at[g] + seats[seat];
        }
    }
    homes
}

/// The smallest distance along `direction` (a unit vector) where a circle of radius `reach`
/// stays [`CLUSTER_GAP`] clear of every `placed` cluster.
fn nearest_free(
    direction: Vec2,
    reach: f32,
    placed: &[usize],
    centres: &[Vec2],
    reaches: &[f32],
) -> f32 {
    // Each placed cluster rules out an interval of distances along the ray.
    let mut blocked: Vec<(f32, f32)> = placed
        .iter()
        .filter_map(|&p| {
            let (centre, clear) = (centres[p], reach + reaches[p] + CLUSTER_GAP);
            let along = direction.dot(centre);
            let discriminant = along * along - centre.length_squared() + clear * clear;
            (discriminant > 0.0).then(|| {
                let half = discriminant.sqrt();
                (along - half, along + half)
            })
        })
        .collect();
    blocked.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut distance = 0.0_f32;
    for (from, to) in blocked {
        if from > distance {
            break;
        }
        distance = distance.max(to);
    }
    distance
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
    fn groups_cluster_around_the_middle() {
        // Group 0: the root (0) and 1. Group 1: 2, its centre, with 3, 4, 7 and 8.
        // Group 2: 5 and 6.
        let groups = [0, 0, 1, 1, 1, 2, 2, 1, 1];
        let links = [
            (0, 2),
            (0, 5),
            (2, 3),
            (2, 4),
            (2, 7),
            (2, 8),
            (5, 6),
            (4, 6),
        ];
        assert_eq!(centres(&groups, &links), [0, 2, 5]);
        let homes = clusters(&groups, &links, 0, &[80.0; 9]);
        assert_eq!(homes[0], Vec2::ZERO);
        assert!(
            (homes[1].length() - PACKING * 80.0).abs() < 0.01,
            "next to the root"
        );
        // Each note is nearer its own centre than the other cluster's.
        for (node, own, other) in [(3, 2, 5), (4, 2, 5), (6, 5, 2), (8, 2, 5)] {
            assert!(homes[node].distance(homes[own]) < homes[node].distance(homes[other]));
        }
        // 4 links into the other cluster: it sits on the side facing it.
        for other in [7, 8] {
            assert!(homes[4].distance(homes[5]) < homes[other].distance(homes[5]));
        }
        // No two notes on the same spot; clusters well clear of each other.
        for a in 0..9 {
            for b in (a + 1)..9 {
                assert!(homes[a].distance(homes[b]) > 30.0, "{a} and {b}");
            }
        }
        assert!(homes[2].distance(homes[5]) > CLUSTER_GAP);
    }

    #[test]
    fn linked_groups_sit_side_by_side() {
        // Four groups of two around a lone root; 1 and 3 link, 2 and 4 do not link at all.
        let groups = [0, 1, 1, 2, 2, 3, 3, 4, 4];
        let links = [(1, 2), (5, 6), (3, 4), (7, 8), (1, 5), (2, 6)];
        let homes = clusters(&groups, &links, 0, &[60.0; 9]);
        let angle = |n: usize| homes[n].to_angle();
        let apart = |a: usize, b: usize| {
            let d = (angle(a) - angle(b)).rem_euclid(TAU);
            d.min(TAU - d)
        };
        assert!(
            apart(1, 5) < apart(1, 3).max(apart(1, 7)),
            "linked neighbours"
        );
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
