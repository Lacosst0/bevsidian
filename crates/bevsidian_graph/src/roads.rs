//! The lines of the star map: one per pair of notes a route joins, drawn the way star charts
//! draw constellations: straight, thin, and stopping short of each star so it shines clear.
//!
//! A route's main line (its core steps) is solid, side quests (optional steps) dashed and
//! secrets (extra steps) dotted. Lines keep their width on screen at any zoom. The followed
//! route burns orange, with a faint wider line under it as its glow. Routes that share a line
//! draw it once. Notes no shown line reaches hang from their parent by a faint dotted trail,
//! and a hovered note shows its links as thin dashes.

use bevy::prelude::*;

use bevsidian_core::{
    nav::{Nav, View},
    vault::{NoteId, StepKind},
};
use bevsidian_ui::color;

use crate::{GraphCamera, GraphNode, GraphState, ortho_scale};

/// A line between two notes, and the routes that take it (and how).
#[derive(Clone, Debug)]
pub struct Road {
    pub a: Entity,
    pub b: Entity,
    pub uses: Vec<(NoteId, StepKind)>,
}

/// The mesh holding every line as quads.
#[derive(Component)]
pub(crate) struct RoadMesh;

/// Widths on screen, in render-target pixels: a main line, a side quest, a secret's dots
/// (their diameter), a trail's dots, a hovered note's links; the followed route's line and
/// its glow.
const MAIN_PX: f32 = 1.6;
const SIDE_PX: f32 = 1.3;
const SECRET_PX: f32 = 2.2;
const TRAIL_PX: f32 = 1.6;
const LINK_PX: f32 = 1.0;
const FOLLOWED_PX: f32 = 2.4;
const FOLLOWED_GLOW_PX: f32 = 6.0;
/// Dashes and gaps of side quests, the gap between dots, in pixels.
const DASH_PX: f32 = 7.0;
const DASH_GAP_PX: f32 = 5.0;
const DOT_GAP_PX: f32 = 5.0;
/// How far short of a star (beyond its outline) a line stops, in pixels.
const GAP_PX: f32 = 4.0;

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

/// The part of the segment from `a` to `b` outside the two stars (`ra` and `rb` around their
/// centres), or `None` when they overlap.
pub fn between(a: Vec2, ra: f32, b: Vec2, rb: f32) -> Option<(Vec2, Vec2)> {
    let length = a.distance(b);
    if length <= ra + rb {
        return None;
    }
    let direction = (b - a) / length;
    Some((a + direction * ra, b - direction * rb))
}

/// Vertices, colours and indices of the line mesh.
#[derive(Default)]
pub(crate) struct LineMesh {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl LineMesh {
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

    fn dashed(&mut self, (from, to): (Vec2, Vec2), half: f32, px: f32, z: f32, color: [f32; 4]) {
        let direction = (to - from).normalize_or_zero();
        for (start, end) in pattern(from.distance(to), DASH_PX * px, DASH_GAP_PX * px) {
            self.quad(
                from + direction * start,
                from + direction * end,
                half,
                z,
                color,
            );
        }
    }

    fn dotted(&mut self, (from, to): (Vec2, Vec2), radius: f32, px: f32, z: f32, color: [f32; 4]) {
        let direction = (to - from).normalize_or_zero();
        let gap = DOT_GAP_PX * px;
        for (start, end) in pattern(from.distance(to), radius * 2.0, gap) {
            self.dot(
                from + direction * f32::midpoint(start, end),
                radius,
                z,
                color,
            );
        }
    }

    /// A line in the style of `kind`, `width` pixels wide; `px` is the world size of a pixel.
    fn line(
        &mut self,
        ends: (Vec2, Vec2),
        kind: StepKind,
        width: f32,
        px: f32,
        z: f32,
        color: Color,
    ) {
        let rgba = color.to_linear().to_f32_array();
        let half = width * px / 2.0;
        match kind {
            StepKind::Core => self.quad(ends.0, ends.1, half, z, rgba),
            StepKind::Optional => self.dashed(ends, half, px, z, rgba),
            StepKind::Extra => self.dotted(ends, SECRET_PX * px / 2.0, px, z, rgba),
        }
    }
}

/// The strongest of two ways of taking a line: a main line beats a side quest beats a secret.
fn stronger(a: StepKind, b: StepKind) -> StepKind {
    let rank = |k: StepKind| match k {
        StepKind::Core => 0,
        StepKind::Optional => 1,
        StepKind::Extra => 2,
    };
    if rank(a) <= rank(b) { a } else { b }
}

/// How a line looks now, if it shows: as the followed route takes it, as the shown routes
/// do, or (with the whole map shown, or while editing) faintly as any route does. The flag
/// says it is the followed route's.
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
    // A star's centre, and how far from it lines stop.
    let star = |e: Entity| {
        nodes
            .get(e)
            .ok()
            .filter(|(_, n)| !n.hidden && !n.tucking)
            .map(|(t, n)| (t.translation.truncate(), n.radius * t.scale.x + GAP_PX * px))
    };
    let mut out = LineMesh::default();
    let mut reached = std::collections::HashSet::new();
    for road in &state.roads {
        let (Some((a, ra)), Some((b, rb))) = (star(road.a), star(road.b)) else {
            continue;
        };
        let Some((kind, mut tint, followed)) = look(&state, road) else {
            continue;
        };
        reached.insert(road.a);
        reached.insert(road.b);
        let Some(ends) = between(a, ra, b, rb) else {
            continue;
        };
        let hovered = focus.is_some_and(|f| f == road.a || f == road.b);
        if hovered && !followed {
            tint = color::MUTED_FOREGROUND;
        }
        if followed {
            // Its glow first, under it.
            out.line(
                ends,
                StepKind::Core,
                FOLLOWED_GLOW_PX,
                px,
                0.0,
                color::MAP_ROUTE_GLOW,
            );
            out.line(ends, kind, FOLLOWED_PX, px, 0.03, tint);
        } else {
            let width = if kind == StepKind::Core {
                MAIN_PX
            } else {
                SIDE_PX
            };
            let z = if hovered { 0.02 } else { 0.01 };
            out.line(ends, kind, width, px, z, tint);
        }
    }
    // Trails to the parent of notes no line reaches (not to the capital: they would cut
    // across the ring).
    for &(child, parent) in &state.trails {
        if reached.contains(&child) || nodes.get(parent).is_ok_and(|(_, n)| n.rank == 0) {
            continue;
        }
        if let (Some((a, ra)), Some((b, rb))) = (star(child), star(parent))
            && let Some(ends) = between(a, ra, b, rb)
        {
            let tint = color::MAP_TRAIL.to_linear().to_f32_array();
            out.dotted(ends, TRAIL_PX * px / 2.0, px, 0.005, tint);
        }
    }
    // A hovered note's links.
    if let Some(focus) = focus
        && let Some((f, rf)) = star(focus)
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
            if let Some((o, ro)) = star(other)
                && let Some(ends) = between(f, rf, o, ro)
            {
                out.dashed(ends, LINK_PX * px / 2.0, px, 0.04, link);
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
    fn lines_stop_short_of_the_stars() {
        let (a, b) = (Vec2::ZERO, Vec2::new(100.0, 0.0));
        assert_eq!(
            between(a, 10.0, b, 20.0),
            Some((Vec2::new(10.0, 0.0), Vec2::new(80.0, 0.0)))
        );
        assert_eq!(between(a, 60.0, b, 50.0), None, "stars that overlap");
    }
}
