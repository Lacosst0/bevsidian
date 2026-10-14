//! The lines of the star map: one per pair of notes a route joins, drawn the way star charts
//! draw constellations: straight, thin, and stopping short of each star so it shines clear.
//!
//! A route's main line (its core steps) is solid, side quests (optional steps) dashed and
//! secrets (extra steps) dotted. Lines keep their width on screen at any zoom. The followed
//! route burns orange, with a faint wider line under it as its glow. Routes that share a line
//! draw it once. Notes no shown line reaches hang from their parent by a faint dotted trail,
//! the capital sends a dotted ray to every region's town, and a hovered note shows its links
//! as thin dashes.

use bevy::prelude::*;

use bevsidian_core::{
    nav::{Nav, View},
    vault::{NoteId, StepKind},
};
use bevsidian_ui::color;

use crate::{GraphCamera, GraphNode, GraphState, mesh::Shapes, neighbors, ortho_scale};

/// A line between two notes, and the routes that take it (and how).
#[derive(Clone, Debug)]
pub struct Line {
    pub a: Entity,
    pub b: Entity,
    pub uses: Vec<(NoteId, StepKind)>,
}

/// The mesh holding every line as quads.
#[derive(Component)]
pub(crate) struct LinesMesh;

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
/// Depths, back to front: trails, the followed route's glow, lines, a hovered note's lines,
/// the followed route, a hovered note's links.
const Z_TRAIL: f32 = -0.01;
const Z_GLOW: f32 = 0.0;
const Z_LINE: f32 = 0.01;
const Z_HOVERED: f32 = 0.02;
const Z_FOLLOWED: f32 = 0.03;
const Z_LINK: f32 = 0.04;

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

fn dashed(out: &mut Shapes, (from, to): (Vec2, Vec2), half: f32, px: f32, z: f32, color: [f32; 4]) {
    let direction = (to - from).normalize_or_zero();
    for (start, end) in pattern(from.distance(to), DASH_PX * px, DASH_GAP_PX * px) {
        out.quad(
            from + direction * start,
            from + direction * end,
            half,
            z,
            color,
        );
    }
}

fn dotted(
    out: &mut Shapes,
    (from, to): (Vec2, Vec2),
    radius: f32,
    px: f32,
    z: f32,
    color: [f32; 4],
) {
    let direction = (to - from).normalize_or_zero();
    for (start, end) in pattern(from.distance(to), radius * 2.0, DOT_GAP_PX * px) {
        let at = from + direction * f32::midpoint(start, end);
        out.disc(at, radius, 8, z, color);
    }
}

/// A line in the style of `kind`, `width` pixels wide; `px` is the world size of a pixel.
fn stroke(
    out: &mut Shapes,
    ends: (Vec2, Vec2),
    kind: StepKind,
    (width, px): (f32, f32),
    z: f32,
    color: Color,
) {
    let rgba = color.to_linear().to_f32_array();
    let half = width * px / 2.0;
    match kind {
        StepKind::Core => out.quad(ends.0, ends.1, half, z, rgba),
        StepKind::Optional => dashed(out, ends, half, px, z, rgba),
        StepKind::Extra => dotted(out, ends, SECRET_PX * px / 2.0, px, z, rgba),
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
pub(crate) fn look(state: &GraphState, line: &Line) -> Option<(StepKind, Color, bool)> {
    let best = |keep: &dyn Fn(NoteId) -> bool| {
        line.uses
            .iter()
            .filter(|(route, _)| keep(*route))
            .map(|&(_, kind)| kind)
            .reduce(stronger)
    };
    if let Some(kind) = best(&|r| Some(r) == state.followed) {
        return Some((kind, color::LINK, true));
    }
    if let Some(kind) = best(&|r| state.active.contains(&r)) {
        return Some((kind, color::MAP_LINE, false));
    }
    if state.whole || state.editing {
        return best(&|_| true).map(|kind| (kind, color::MAP_LINE_FAINT, false));
    }
    None
}

pub(crate) fn draw_lines(
    nav: Res<Nav>,
    state: Res<GraphState>,
    nodes: Query<(&Transform, &GraphNode)>,
    camera: Query<&Projection, With<GraphCamera>>,
    line_mesh: Query<&Mesh2d, With<LinesMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if nav.view.get_untracked() != View::Graph {
        return;
    }
    let (Ok(mesh), Ok(projection)) = (line_mesh.single(), camera.single()) else {
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
    let mut out = Shapes::default();
    let mut reached = std::collections::HashSet::new();
    for line in &state.lines {
        let (Some((a, ra)), Some((b, rb))) = (star(line.a), star(line.b)) else {
            continue;
        };
        let Some((kind, mut tint, followed)) = look(&state, line) else {
            continue;
        };
        reached.insert(line.a);
        reached.insert(line.b);
        let Some(ends) = between(a, ra, b, rb) else {
            continue;
        };
        let hovered = focus.is_some_and(|f| f == line.a || f == line.b);
        if hovered && !followed {
            tint = color::MUTED_FOREGROUND;
        }
        if followed {
            // Its glow first, under it.
            stroke(
                &mut out,
                ends,
                StepKind::Core,
                (FOLLOWED_GLOW_PX, px),
                Z_GLOW,
                color::MAP_ROUTE_GLOW,
            );
            stroke(&mut out, ends, kind, (FOLLOWED_PX, px), Z_FOLLOWED, tint);
        } else {
            let width = if kind == StepKind::Core {
                MAIN_PX
            } else {
                SIDE_PX
            };
            let z = if hovered { Z_HOVERED } else { Z_LINE };
            stroke(&mut out, ends, kind, (width, px), z, tint);
        }
    }
    // Trails to the parent of notes no line reaches, and rays from the capital to every
    // region's town: the capital links to them all.
    for &(child, parent) in &state.trails {
        let to_capital = nodes.get(parent).is_ok_and(|(_, n)| n.rank == 0);
        let ray = to_capital && nodes.get(child).is_ok_and(|(_, n)| n.rank == 1);
        if !ray && (reached.contains(&child) || to_capital) {
            continue;
        }
        if let (Some((a, ra)), Some((b, rb))) = (star(child), star(parent))
            && let Some(ends) = between(a, ra, b, rb)
        {
            let tint = color::MAP_TRAIL.to_linear().to_f32_array();
            dotted(&mut out, ends, TRAIL_PX * px / 2.0, px, Z_TRAIL, tint);
        }
    }
    // A hovered note's links.
    if let Some(focus) = focus
        && let Some((f, rf)) = star(focus)
    {
        let link = color::MAP_LINK.to_linear().to_f32_array();
        for other in neighbors(&state, focus) {
            if let Some((o, ro)) = star(other)
                && let Some(ends) = between(f, rf, o, ro)
            {
                dashed(&mut out, ends, LINK_PX * px / 2.0, px, Z_LINK, link);
            }
        }
    }
    out.upload(&mut meshes, &mesh.0);
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
