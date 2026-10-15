//! The lines of the map: one per pair of notes a route joins, straight, stopping short of
//! each note.
//!
//! A route's core steps are joined by a solid line, its optional steps (`(optional)` and
//! `(extra)`) by dotted ones; the line a route leaves the main quest by is dashed: there you
//! choose your path. The main quest is one solid white line; every other route has its own
//! colour from its first page on. Lines keep their width on screen at any zoom; the followed
//! route is thicker, with a faint wider line under it as its glow. Routes that share a line
//! draw it once. A note no route takes is tied to a note near it by a thin line in that
//! note's route colour. A hovered note shows its links as thin lines.

use bevy::prelude::*;

use bevsidian_core::{
    nav::{Nav, View},
    vault::{NoteId, StepKind},
};
use bevsidian_ui::color;

use crate::{
    GraphCamera, GraphNode, GraphState, mesh::Shapes, neighbors, ortho_scale, routes::Use,
};

/// A line between two notes, and the routes that take it (and how).
#[derive(Clone, Debug)]
pub struct Line {
    pub a: Entity,
    pub b: Entity,
    /// The routes that take it; none for a tie (a note no route takes, to a note near it).
    pub uses: Vec<Use>,
    /// The route that leaves the main quest by this line: it is grey, as the main quest, up
    /// to the route's first page.
    pub start: Option<NoteId>,
    /// A tie: a note no route takes, to a note near it (thin, with no reading order).
    pub tie: bool,
}

/// The mesh holding every line as quads.
#[derive(Component)]
pub(crate) struct LinesMesh;

/// Widths on screen, in render-target pixels: a core line, an optional step's dots (their
/// diameter), a hovered note's links; the followed route's line and its glow.
const MAIN_PX: f32 = 2.2;
const DOT_PX: f32 = 2.6;
const LINK_PX: f32 = 1.0;
/// A tie's width.
const TIE_PX: f32 = 1.3;
const FOLLOWED_PX: f32 = 3.2;
const FOLLOWED_GLOW_PX: f32 = 8.0;
/// Dashes and gaps of a line leaving the main quest (where you choose your path), the gap
/// between dots, in pixels.
const DASH_PX: f32 = 9.0;
const DASH_GAP_PX: f32 = 6.0;
const DOT_GAP_PX: f32 = 5.0;
/// How far short of a note (beyond its shape) a line stops, in pixels.
const GAP_PX: f32 = 4.0;
/// Depths, back to front: the followed route's glow, lines, a hovered note's lines, the
/// followed route, a hovered note's links.
/// How far a route not shown fades toward the sky with the whole map.
pub(crate) const WHOLE_DIM: f32 = 0.2;
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

/// The part of the segment from `a` to `b` outside the two notes (`ra` and `rb` around their
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

/// How a line is drawn: solid along a route, dashed where you choose your path (leaving the
/// main quest), dotted to an optional step.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Style {
    Solid,
    Dashed,
    Dotted,
}

/// A line in `style`, `width` pixels wide; `px` is the world size of a pixel.
fn stroke(
    out: &mut Shapes,
    ends: (Vec2, Vec2),
    style: Style,
    (width, px): (f32, f32),
    z: f32,
    color: Color,
) {
    let rgba = color.to_linear().to_f32_array();
    let half = width * px / 2.0;
    match style {
        Style::Solid => out.quad(ends.0, ends.1, half, z, rgba),
        Style::Dashed => dashed(out, ends, half, px, z, rgba),
        Style::Dotted => dotted(out, ends, DOT_PX * px / 2.0, px, z, rgba),
    }
}

/// The strongest of two ways of taking a line: a core step beats an optional one beats an
/// extra one.
fn stronger(a: StepKind, b: StepKind) -> StepKind {
    let rank = |k: StepKind| match k {
        StepKind::Core => 0,
        StepKind::Optional => 1,
        StepKind::Extra => 2,
    };
    if rank(a) <= rank(b) { a } else { b }
}

/// How a line looks now, if it shows.
pub(crate) struct Look {
    kind: StepKind,
    route: NoteId,
    tint: Color,
    /// The tint of the line a route leaves the main quest by (as the main quest's).
    lead: Color,
    /// It is the followed route's.
    followed: bool,
    /// It is a tie: thin, with no reading order.
    tie: bool,
}

/// How a line looks now, if it shows: as the followed route takes it, as the shown routes
/// do (the main quest's white where it is one of them, else the first route's colour), or,
/// with the whole map shown or while editing, as any route does.
pub(crate) fn look(state: &GraphState, line: &Line) -> Option<Look> {
    if line.uses.is_empty() {
        return Some(Look {
            kind: StepKind::Core,
            route: NoteId::MAX,
            tint: color::MAP_LINE_FAINT,
            lead: color::MAP_LINE_FAINT,
            followed: false,
            tie: true,
        });
    }
    let pick = |keep: &dyn Fn(&Use) -> bool| {
        line.uses
            .iter()
            .filter(|u| keep(u))
            .copied()
            .reduce(|best, u| {
                if stronger(u.kind, best.kind) == u.kind && u.kind != best.kind {
                    u
                } else {
                    best
                }
            })
    };
    let colour = |u: &Use| {
        state
            .colors
            .get(&u.route)
            .copied()
            .unwrap_or(color::MAP_LINE)
    };
    if let Some(u) = pick(&|u| Some(u.route) == state.followed) {
        return Some(Look {
            kind: u.kind,
            route: u.route,
            tint: colour(&u),
            lead: color::MAP_LINE,
            followed: true,
            tie: line.tie,
        });
    }
    let main = |u: &Use| colour(u) == color::MAP_LINE;
    if let Some(u) = pick(&|u| state.active.contains(&u.route) && main(u))
        .or_else(|| pick(&|u| state.active.contains(&u.route)))
    {
        return Some(Look {
            kind: u.kind,
            route: u.route,
            tint: colour(&u),
            lead: color::MAP_LINE,
            followed: false,
            tie: line.tie,
        });
    }
    if state.whole || state.editing {
        // Every route in its colour, dimmed under the ones shown.
        return pick(&|_| true).map(|u| Look {
            kind: u.kind,
            route: u.route,
            lead: color::MAP_LINE,
            tint: if main(&u) {
                color::MAP_LINE
            } else {
                colour(&u).mix(&color::BACKGROUND, WHOLE_DIM)
            },
            followed: false,
            tie: line.tie,
        });
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
    if nav.view != View::Graph {
        return;
    }
    let (Ok(mesh), Ok(projection)) = (line_mesh.single(), camera.single()) else {
        return;
    };
    // World units per render-target pixel.
    let px = ortho_scale(projection);
    let focus = state.hovered.or(state.dragging);
    // A note's centre, and how far from it lines stop.
    let end = |e: Entity| {
        nodes
            .get(e)
            .ok()
            .filter(|(_, n)| !n.hidden)
            .map(|(t, n)| (t.translation.truncate(), n.radius * t.scale.x + GAP_PX * px))
    };
    let mut out = Shapes::default();
    for line in &state.lines {
        let (Some((a, ra)), Some((b, rb))) = (end(line.a), end(line.b)) else {
            continue;
        };
        let Some(look) = look(&state, line) else {
            continue;
        };
        let Some((from, to)) = between(a, ra, b, rb) else {
            continue;
        };
        let ends = (from, to);
        // The line a route leaves the main quest by is the main quest's colour, dashed: there
        // you choose your path. The route's colour begins at its first page.
        let leaves = line.start == Some(look.route);
        let style = if leaves {
            Style::Dashed
        } else if look.kind == StepKind::Core {
            Style::Solid
        } else {
            Style::Dotted
        };
        let hovered = focus.is_some_and(|f| f == line.a || f == line.b);
        let mut tint = if leaves { look.lead } else { look.tint };
        if hovered && !look.followed && tint == color::MAP_LINE_FAINT {
            tint = color::MUTED_FOREGROUND;
        }
        let (width, z) = if look.followed {
            // Its glow first, under it.
            let glow = tint.mix(&color::BACKGROUND, 0.7);
            stroke(
                &mut out,
                ends,
                Style::Solid,
                (FOLLOWED_GLOW_PX, px),
                Z_GLOW,
                glow,
            );
            (FOLLOWED_PX, Z_FOLLOWED)
        } else if look.tie {
            (TIE_PX, if hovered { Z_HOVERED } else { Z_LINE })
        } else {
            (MAIN_PX, if hovered { Z_HOVERED } else { Z_LINE })
        };
        stroke(&mut out, ends, style, (width, px), z, tint);
    }
    // A hovered note's links.
    if let Some(focus) = focus
        && let Some((f, rf)) = end(focus)
    {
        let link = color::MAP_LINK.to_linear().to_f32_array();
        for other in neighbors(&state, focus) {
            if let Some((o, ro)) = end(other)
                && let Some(ends) = between(f, rf, o, ro)
            {
                out.quad(ends.0, ends.1, LINK_PX * px / 2.0, Z_LINK, link);
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
