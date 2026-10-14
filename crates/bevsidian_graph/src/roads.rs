//! The roads of the map: one per pair of notes a route joins, drawn as a gentle curve.
//!
//! Main roads (a route's core steps) are solid, side quests (optional steps) dashed and
//! secrets (extra steps) dotted. Main roads and side quests have a casing of land, so over
//! the sea they run on a causeway. A road's bend depends only on its two ends, so routes that
//! share a road draw it once. Notes that no shown road reaches hang from their parent by a
//! faint trail, and a hovered note shows its links as thin lines.

use bevy::prelude::*;

use bevsidian_core::{
    nav::{Nav, View},
    vault::{NoteId, StepKind},
};
use bevsidian_ui::color;

use crate::{GraphCamera, GraphNode, GraphState, ortho_scale};

/// A road between two notes, and the routes that take it (and how).
#[derive(Clone, Debug)]
pub struct Road {
    pub a: Entity,
    pub b: Entity,
    /// Which way it bends (+1 or −1): fixed by its two notes.
    pub bend: f32,
    pub uses: Vec<(NoteId, StepKind)>,
}

/// The mesh holding every road as quads.
#[derive(Component)]
pub(crate) struct RoadMesh;

/// How much a road bends: its control point sits this share of its length to one side.
const BEND: f32 = 0.12;
/// Widths in world units: main roads, side quests, the dots of secrets, trails, link lines.
const MAIN_WIDTH: f32 = 9.0;
const SIDE_WIDTH: f32 = 6.0;
const SECRET_DOT: f32 = 3.6;
const TRAIL_DOT: f32 = 2.0;
const LINK_WIDTH: f32 = 1.5;
/// The casing around a road, on each side.
const CASING: f32 = 3.0;
/// Dashes and gaps of side quests, the gap between dots, in world units.
const DASH: f32 = 18.0;
const DASH_GAP: f32 = 11.0;
const DOT_GAP: f32 = 13.0;
/// Thinnest a line gets on screen, in render-target pixels.
const MIN_PX: f32 = 1.4;

/// Which way the road between notes `a` and `b` bends: the same whichever end comes first.
pub fn bend_of(a: NoteId, b: NoteId) -> f32 {
    if (a.min(b) * 7 + a.max(b)).is_multiple_of(2) {
        1.0
    } else {
        -1.0
    }
}

/// Points along the curve from `a` to `b`, bending to the left of `a → b` for a positive
/// `bend` (the curve is the same, reversed, from `b` to `a` with the bend negated).
pub fn curve(a: Vec2, b: Vec2, bend: f32) -> Vec<Vec2> {
    let delta = b - a;
    let control = (a + b) / 2.0 + delta.perp() * BEND * bend;
    let segments = (delta.length() / 40.0).clamp(4.0, 24.0) as usize;
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            a.lerp(control, t).lerp(control.lerp(b, t), t)
        })
        .collect()
}

/// The stretch of a polyline from arc length `from` to `to`.
fn stretch(points: &[Vec2], from: f32, to: f32) -> Vec<Vec2> {
    let mut out = Vec::new();
    let mut walked = 0.0;
    for pair in points.windows(2) {
        let length = pair[0].distance(pair[1]);
        let (start, end) = (walked, walked + length);
        walked = end;
        if end < from || start > to || length == 0.0 {
            continue;
        }
        let at = |s: f32| pair[0].lerp(pair[1], ((s - start) / length).clamp(0.0, 1.0));
        if out.is_empty() {
            out.push(at(from.max(start)));
        }
        out.push(at(to.min(end)));
    }
    out
}

fn length(points: &[Vec2]) -> f32 {
    points.windows(2).map(|p| p[0].distance(p[1])).sum()
}

/// Pieces `piece` long, `gap` apart, centred on `0..length` (start and end of each).
pub fn pattern(length: f32, piece: f32, gap: f32) -> impl Iterator<Item = (f32, f32)> {
    let count = ((length + gap) / (piece + gap)).floor().max(0.0) as usize;
    let used = count as f32 * (piece + gap) - gap;
    let start = (length - used) / 2.0;
    (0..count).map(move |i| {
        let from = start + i as f32 * (piece + gap);
        (from, from + piece)
    })
}

/// Vertices, colours and indices of the road mesh.
#[derive(Default)]
pub(crate) struct LineMesh {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl LineMesh {
    /// A solid line along `points`, `half` wide on each side, at depth `z`.
    fn solid(&mut self, points: &[Vec2], half: f32, z: f32, color: [f32; 4]) {
        for pair in points.windows(2) {
            self.quad(pair[0], pair[1], half, z, color);
        }
        // Round joints, so bends have no notches.
        for &p in &points[1..points.len().saturating_sub(1)] {
            self.dot(p, half, z, color);
        }
    }

    fn dashed(&mut self, points: &[Vec2], half: f32, z: f32, color: [f32; 4]) {
        for (from, to) in pattern(length(points), DASH, DASH_GAP) {
            self.solid(&stretch(points, from, to), half, z, color);
        }
    }

    fn dotted(&mut self, points: &[Vec2], radius: f32, gap: f32, z: f32, color: [f32; 4]) {
        for (from, to) in pattern(length(points), radius * 2.0, gap) {
            let middle = f32::midpoint(from, to);
            if let Some(&p) = stretch(points, middle, middle).first() {
                self.dot(p, radius, z, color);
            }
        }
    }

    fn quad(&mut self, from: Vec2, to: Vec2, half: f32, z: f32, color: [f32; 4]) {
        let normal = (to - from).normalize_or_zero().perp() * half;
        let base = self.positions.len() as u32;
        for p in [from + normal, from - normal, to - normal, to + normal] {
            self.positions.push([p.x, p.y, z]);
            self.colors.push(color);
        }
        self.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// A round dot (an octagon).
    fn dot(&mut self, center: Vec2, radius: f32, z: f32, color: [f32; 4]) {
        let base = self.positions.len() as u32;
        self.positions.push([center.x, center.y, z]);
        self.colors.push(color);
        for i in 0..8 {
            let p = center + Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4) * radius;
            self.positions.push([p.x, p.y, z]);
            self.colors.push(color);
            self.indices
                .extend([base, base + 1 + i, base + 1 + (i + 1) % 8]);
        }
    }

    /// A road in the style of `kind`, with its casing below it.
    fn road(&mut self, points: &[Vec2], kind: StepKind, px: f32, color: Color, lifted: bool) {
        let rgba = color.to_linear().to_f32_array();
        let casing = color::MAP_LAND.to_linear().to_f32_array();
        let min = MIN_PX * px / 2.0;
        // Lifted roads (the followed route, a hovered note's) draw over the others.
        let z = if lifted { 0.02 } else { 0.0 };
        match kind {
            StepKind::Core => {
                let half = (MAIN_WIDTH / 2.0).max(min);
                self.solid(points, half + CASING, z, casing);
                self.solid(points, half, z + 0.01, rgba);
            }
            StepKind::Optional => {
                let half = (SIDE_WIDTH / 2.0).max(min);
                self.solid(points, half + CASING, z, casing);
                self.dashed(points, half, z + 0.01, rgba);
            }
            StepKind::Extra => {
                let radius = SECRET_DOT.max(min);
                self.dotted(points, radius, DOT_GAP, z + 0.01, rgba);
            }
        }
    }
}

/// The strongest of two ways of taking a road: a main road beats a side quest beats a secret.
fn stronger(a: StepKind, b: StepKind) -> StepKind {
    let rank = |k: StepKind| match k {
        StepKind::Core => 0,
        StepKind::Optional => 1,
        StepKind::Extra => 2,
    };
    if rank(a) <= rank(b) { a } else { b }
}

/// How a road looks now, if it shows: as the followed route takes it, as the shown routes
/// do, or (with the whole map shown) faintly as any route does.
pub(crate) fn look(state: &GraphState, road: &Road) -> Option<(StepKind, Color, bool)> {
    let best = |keep: &dyn Fn(NoteId) -> bool| {
        road.uses
            .iter()
            .filter(|(route, _)| keep(*route))
            .map(|&(_, kind)| kind)
            .reduce(stronger)
    };
    if let Some(kind) = best(&|r| Some(r) == state.followed) {
        return Some((kind, color::LINK, true));
    }
    if let Some(kind) = best(&|r| state.active.contains(&r)) {
        return Some((kind, color::MAP_ROAD, false));
    }
    if state.whole || state.editing {
        return best(&|_| true).map(|kind| (kind, color::MAP_ROAD_FAINT, false));
    }
    None
}

pub(crate) fn draw_roads(
    nav: Res<Nav>,
    state: Res<GraphState>,
    nodes: Query<(&Transform, &GraphNode)>,
    camera: Query<&Projection, With<GraphCamera>>,
    road_mesh: Query<&Mesh2d, With<RoadMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if nav.view.get_untracked() != View::Graph {
        return;
    }
    let (Ok(mesh), Ok(projection)) = (road_mesh.single(), camera.single()) else {
        return;
    };
    // World units per render-target pixel.
    let px = ortho_scale(projection);
    let focus = state.hovered.or(state.dragging);
    let shown = |e: Entity| nodes.get(e).ok().filter(|(_, n)| !n.hidden && !n.tucking);
    let mut out = LineMesh::default();
    let mut reached = std::collections::HashSet::new();
    for road in &state.roads {
        let (Some((ta, _)), Some((tb, _))) = (shown(road.a), shown(road.b)) else {
            continue;
        };
        let Some((kind, mut tint, mut lifted)) = look(&state, road) else {
            continue;
        };
        reached.insert(road.a);
        reached.insert(road.b);
        if focus.is_some_and(|f| f == road.a || f == road.b) && tint != color::LINK {
            tint = color::MUTED_FOREGROUND;
            lifted = true;
        }
        let points = curve(
            ta.translation.truncate(),
            tb.translation.truncate(),
            road.bend,
        );
        out.road(&points, kind, px, tint, lifted);
    }
    // Trails to the parent of notes no road reaches.
    let trail = color::MAP_TRAIL.to_linear().to_f32_array();
    for &(child, parent) in &state.trails {
        // Trails to the capital would cut across the ring road.
        if reached.contains(&child) || nodes.get(parent).is_ok_and(|(_, n)| n.rank == 0) {
            continue;
        }
        if let (Some((tc, nc)), Some((tp, _))) = (shown(child), shown(parent)) {
            let points = curve(tc.translation.truncate(), tp.translation.truncate(), 0.0);
            let radius = TRAIL_DOT.max(MIN_PX * px / 2.0);
            out.dotted(
                &stretch(&points, nc.radius, f32::MAX),
                radius,
                DOT_GAP,
                -0.01,
                trail,
            );
        }
    }
    // A hovered note's links.
    if let Some(focus) = focus
        && let Some((tf, _)) = shown(focus)
    {
        let link = color::LINK_SOFT_SOLID.to_linear().to_f32_array();
        for &(a, b) in &state.edges {
            let other = if a == focus {
                b
            } else if b == focus {
                a
            } else {
                continue;
            };
            if let Some((to, _)) = shown(other) {
                let points = [tf.translation.truncate(), to.translation.truncate()];
                out.dashed(
                    &points,
                    (LINK_WIDTH / 2.0).max(MIN_PX * px / 2.0),
                    0.03,
                    link,
                );
            }
        }
    }
    if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, out.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, out.colors);
        mesh.insert_indices(bevy::mesh::Indices::U32(out.indices));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns_are_centred() {
        let pieces: Vec<_> = pattern(50.0, 10.0, 5.0).collect();
        assert_eq!(pieces, vec![(5.0, 15.0), (20.0, 30.0), (35.0, 45.0)]);
        assert_eq!(pattern(5.0, 10.0, 5.0).count(), 0);
    }

    #[test]
    fn a_road_is_the_same_curve_from_either_end() {
        let (a, b) = (Vec2::ZERO, Vec2::new(400.0, 0.0));
        let there = curve(a, b, bend_of(3, 8));
        let mut back = curve(b, a, -bend_of(8, 3));
        back.reverse();
        for (p, q) in there.iter().zip(&back) {
            assert!(p.distance(*q) < 0.01, "{p} {q}");
        }
        // It bends: its middle is off the straight line, by the share it should.
        let middle = there[there.len() / 2];
        assert!(
            (middle.y.abs() - 400.0 * BEND / 2.0).abs() < 1.0,
            "{middle}"
        );
    }

    #[test]
    fn stretches_follow_the_curve() {
        let points = [Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        assert_eq!(length(&points), 20.0);
        assert_eq!(
            stretch(&points, 5.0, 15.0),
            vec![
                Vec2::new(5.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(10.0, 5.0)
            ]
        );
    }
}
