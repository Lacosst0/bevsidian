//! Routes on the map: the reading paths drawn as lines between their notes.
//!
//! The main quest (`quest: main`) starts at the hub; every other route starts at the stop of
//! the main quest nearest its first step (or at the hub), so lines branch off the spiral
//! instead of all crossing the hub. A route's core steps are joined in order; each side quest
//! or secret hangs from the route's core step nearest it, so a spur is always short. The map
//! shows the main quest, the routes chosen in its checklist and the route being followed.
//! A note no route takes (mostly crates) is tied to a note near it that it links with
//! ([`ties`]), so nothing floats unconnected.

use bevy::prelude::*;

use bevsidian_core::{
    nav::Nav,
    vault::{NoteId, Quest, StepKind, Vault, VaultData},
};

use crate::{GraphLabel, GraphNode, GraphState, label_text};

/// How a route takes a line: which route, as what kind of step, and whether it goes from the
/// line's lower note to its higher one (the reading order, for the line's arrow).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Use {
    pub route: NoteId,
    pub kind: StepKind,
    pub forward: bool,
}

/// A line's two notes (lower id first), and the routes that take it.
pub type LineEnds = (NoteId, NoteId, Vec<Use>);

/// Every route's lines, and where each route but the main quest begins (the line its bullet
/// sits on: from the main quest to its first step, or its first two steps).
pub struct RouteLines {
    pub lines: Vec<LineEnds>,
    pub starts: Vec<(NoteId, NoteId, NoteId)>,
}

/// The lines of every route. `place` says where a note sits; `hub` is where the main quest
/// starts.
pub fn lines(vault: &VaultData, place: &dyn Fn(NoteId) -> Option<Vec2>, hub: NoteId) -> RouteLines {
    let at = |n: NoteId| place(n).unwrap_or_default();
    let nearest = |to: NoteId, among: &[NoteId]| {
        among
            .iter()
            .copied()
            .filter(|&a| a != to)
            .min_by(|&a, &b| at(a).distance(at(to)).total_cmp(&at(b).distance(at(to))))
    };
    let mut anchors = vec![hub];
    for path in vault.paths.iter().filter(|p| p.quest == Quest::Main) {
        for (&step, &kind) in path.steps.iter().zip(&path.kinds) {
            if kind == StepKind::Core && !anchors.contains(&step) {
                anchors.push(step);
            }
        }
    }
    let mut out: Vec<LineEnds> = Vec::new();
    let mut add = |from: NoteId, to: NoteId, route: NoteId, kind: StepKind| {
        if from == to {
            return;
        }
        let (a, b) = (from.min(to), from.max(to));
        let index = out
            .iter()
            .position(|r| (r.0, r.1) == (a, b))
            .unwrap_or_else(|| {
                out.push((a, b, Vec::new()));
                out.len() - 1
            });
        let line = &mut out[index];
        let taken = Use {
            route,
            kind,
            forward: from == a,
        };
        match line.2.iter_mut().find(|u| u.route == route) {
            // A route taking a line twice takes it as a core step if either is.
            Some(was) if kind == StepKind::Core => *was = taken,
            Some(_) => {}
            None => line.2.push(taken),
        }
    };
    let mut starts = Vec::new();
    for path in &vault.paths {
        let Some(&first) = path.steps.first() else {
            continue;
        };
        let start = if path.quest == Quest::Main {
            Some(hub)
        } else if anchors.contains(&first) {
            None
        } else {
            nearest(first, &anchors)
        };
        let cores: Vec<NoteId> = path
            .steps
            .iter()
            .zip(&path.kinds)
            .filter(|(_, k)| **k == StepKind::Core)
            .map(|(&s, _)| s)
            .collect();
        if let Some(start) = start {
            add(start, first, path.note, StepKind::Core);
        }
        if path.quest != Quest::Main {
            match (start, cores.get(1)) {
                (Some(start), _) => starts.push((path.note, start, first)),
                (None, Some(&second)) => starts.push((path.note, first, second)),
                (None, None) => {}
            }
        }
        for pair in cores.windows(2) {
            add(pair[0], pair[1], path.note, StepKind::Core);
        }
        for (&step, &kind) in path.steps.iter().zip(&path.kinds) {
            if kind != StepKind::Core
                && let Some(from) = nearest(step, &cores)
            {
                add(from, step, path.note, kind);
            }
        }
    }
    RouteLines { lines: out, starts }
}

/// How far a note no route takes may be tied to a note it links with; past it, to the
/// nearest note.
const TIE_REACH: f32 = 320.0;

/// Ties for the `notes` no line reaches (`connected` are those it does): each to a note
/// already tied in, growing out from the routes like a tree, the shortest tie first. A note
/// is tied to the nearest one it links with (either way) within [`TIE_REACH`], an inner one
/// (a crate to the crates it is compared in, rather than to another crate) first, or to the
/// nearest note at all. Each tie is (the note, the note it hangs from).
pub fn ties(
    vault: &VaultData,
    place: &dyn Fn(NoteId) -> Option<Vec2>,
    notes: &[NoteId],
    connected: &std::collections::HashSet<NoteId>,
) -> Vec<(NoteId, NoteId)> {
    let linked = |a: NoteId, b: NoteId| {
        vault.notes[a].outgoing.contains(&b) || vault.notes[b].outgoing.contains(&a)
    };
    let mut tied: Vec<NoteId> = notes
        .iter()
        .copied()
        .filter(|n| connected.contains(n))
        .collect();
    let mut loose: Vec<NoteId> = notes
        .iter()
        .copied()
        .filter(|n| !connected.contains(n) && place(*n).is_some())
        .collect();
    if tied.is_empty() {
        return Vec::new();
    }
    let order = |a: (bool, bool, f32), b: (bool, bool, f32)| {
        (a.0, a.1).cmp(&(b.0, b.1)).then(a.2.total_cmp(&b.2))
    };
    let mut out = Vec::new();
    while !loose.is_empty() {
        // The best tie of each loose note: linked and in reach first, then the shortest.
        let best = loose
            .iter()
            .enumerate()
            .filter_map(|(i, &n)| {
                let at = place(n)?;
                tied.iter()
                    .filter_map(|&t| {
                        let distance = place(t)?.distance(at);
                        let far = !(linked(n, t) && distance <= TIE_REACH);
                        let sideways = crate::tree::rank(vault, t) >= crate::tree::rank(vault, n);
                        Some(((far, sideways, distance), i, t))
                    })
                    .min_by(|a, b| order(a.0, b.0))
            })
            .min_by(|a, b| order(a.0, b.0));
        let Some((_, i, to)) = best else {
            break;
        };
        let note = loose.swap_remove(i);
        out.push((note, to));
        tied.push(note);
    }
    out
}

/// The routes shown: the main quest, the ones chosen in the checklist and the followed one.
pub fn shown_routes(
    vault: &VaultData,
    chosen: Option<&[NoteId]>,
    followed: Option<NoteId>,
) -> Vec<NoteId> {
    vault
        .paths
        .iter()
        .filter(|p| {
            p.quest == Quest::Main
                || chosen.is_some_and(|c| c.contains(&p.note))
                || followed == Some(p.note)
        })
        .map(|p| p.note)
        .collect()
}

/// Applies the routes chosen, the followed one and "whole map": which notes must show, and
/// the followed route's step numbers on their labels.
pub(crate) fn choose_routes(
    nav: Res<Nav>,
    vault: Res<Vault>,
    mut state: ResMut<GraphState>,
    mut nodes: Query<(&mut GraphNode, &Children)>,
    mut labels: Query<&mut Text2d, With<GraphLabel>>,
    mut last: Local<Option<(Option<Vec<NoteId>>, Option<NoteId>, bool, u64, usize)>>,
) {
    let data = vault.get_untracked();
    let chosen = nav.quests.get_untracked();
    let followed = nav.path.get_untracked().filter(|&p| data.path(p).is_some());
    let whole = nav.whole_map.get_untracked();
    let key = (
        chosen.clone(),
        followed,
        whole,
        data.revision,
        nodes.iter().len(),
    );
    if last.as_ref() == Some(&key) {
        return;
    }
    let first = last.is_none();
    let refollowed = last.as_ref().is_some_and(|l| l.1 != followed);
    // Only a change of routes moves the camera, not the map being rebuilt (after an edit).
    let rerouted = last
        .as_ref()
        .is_some_and(|l| (&l.0, l.1, l.2) != (&chosen, followed, whole));
    *last = Some(key);
    let active = shown_routes(&data, chosen.as_deref(), followed);
    let forced: Vec<NoteId> = active
        .iter()
        .filter(|_| !whole)
        .chain(followed.iter())
        .filter_map(|&p| data.path(p))
        .flat_map(|p| p.steps.iter().copied())
        .collect();
    let steps = followed
        .and_then(|p| data.path(p))
        .map(|p| p.steps.clone())
        .unwrap_or_default();
    for (mut node, children) in &mut nodes {
        node.forced = node.rank == 0 || forced.contains(&node.note);
        let step = steps.iter().position(|&s| s == node.note);
        node.step = step;
        let label = label_text(&data.notes[node.note].name, step);
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child)
                && text.0 != label
            {
                text.0.clone_from(&label);
            }
        }
    }
    state.active = active;
    state.followed = followed;
    state.whole = whole;
    if !first && rerouted {
        state.framing = true;
        state.frame_route = refollowed && followed.is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_branch_off_the_main_quest() {
        let vault = VaultData::from_sources([
            ("Hub.md", "# Hub"),
            ("A.md", "# A"),
            ("B.md", "# B"),
            ("C.md", "# C"),
            ("D.md", "# D"),
            (
                "Main.md",
                "---\npath: true\nquest: main\n---\n# Main\n1. [[A]]\n2. [[B]]\n3. [[C]] (optional)",
            ),
            (
                "Side.md",
                "---\npath: true\nquest: feature\n---\n# Side\n1. [[D]]\n2. [[C]] (extra)",
            ),
        ]);
        let id = |name: &str| vault.find(name).unwrap();
        let place = |n: NoteId| {
            Some(match vault.notes[n].name.as_str() {
                "A" => Vec2::new(100.0, 0.0),
                "B" => Vec2::new(0.0, 100.0),
                "C" => Vec2::new(20.0, 160.0),
                "D" => Vec2::new(0.0, 300.0),
                _ => Vec2::ZERO,
            })
        };
        let drawn = lines(&vault, &place, id("Hub"));
        // How each route takes the line from `a` to `b`, and whether in that direction.
        let line = |a: &str, b: &str| {
            let (lo, hi) = (id(a).min(id(b)), id(a).max(id(b)));
            drawn
                .lines
                .iter()
                .find(|r| (r.0, r.1) == (lo, hi))
                .map(|r| {
                    r.2.iter()
                        .map(|u| (u.route, u.kind, u.forward == (id(a) == lo)))
                        .collect::<Vec<_>>()
                })
        };
        let (main, side) = (id("Main"), id("Side"));
        assert_eq!(line("Hub", "A"), Some(vec![(main, StepKind::Core, true)]));
        assert_eq!(line("A", "B"), Some(vec![(main, StepKind::Core, true)]));
        // A side quest hangs from the route's core step nearest it.
        assert_eq!(line("B", "C"), Some(vec![(main, StepKind::Optional, true)]));
        assert_eq!(line("A", "C"), None);
        // The side route starts at B, the main stop nearest its first step; its bullet sits
        // on that line.
        assert_eq!(line("B", "D"), Some(vec![(side, StepKind::Core, true)]));
        assert_eq!(line("D", "C"), Some(vec![(side, StepKind::Extra, true)]));
        assert_eq!(line("Hub", "D"), None);
        assert_eq!(drawn.starts, vec![(side, id("B"), id("D"))]);
        assert_eq!(shown_routes(&vault, None, None), vec![main]);
        assert_eq!(shown_routes(&vault, Some(&[side]), None), vec![main, side]);
    }
}
