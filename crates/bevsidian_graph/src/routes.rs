//! Routes on the map: the reading paths drawn as roads.
//!
//! The main quest (`quest: main`) starts at the hub; every other route starts at the stop of
//! the main quest nearest its first step (or at the hub), so roads branch off the ring road
//! instead of all crossing the capital. The map shows the main quest, the routes chosen in
//! its checklist and the route being followed: their steps are always on the map (whatever is
//! folded), and every other note only when unfolded, or with the whole map shown.

use bevy::prelude::*;

use bevsidian_core::{
    nav::Nav,
    vault::{NoteId, Quest, StepKind, Vault, VaultData},
};

use crate::{GraphLabel, GraphNode, GraphState};

/// A road's two notes (lower id first), and the routes that take it, with how.
pub type RoadEnds = (NoteId, NoteId, Vec<(NoteId, StepKind)>);

/// The roads of every route. `place` says where a note sits; `hub` is where the main quest
/// starts.
pub fn roads(
    vault: &VaultData,
    place: &dyn Fn(NoteId) -> Option<Vec2>,
    hub: NoteId,
) -> Vec<RoadEnds> {
    let mut anchors = vec![hub];
    for path in vault.paths.iter().filter(|p| p.quest == Quest::Main) {
        for (&step, &kind) in path.steps.iter().zip(&path.kinds) {
            if kind == StepKind::Core && !anchors.contains(&step) {
                anchors.push(step);
            }
        }
    }
    let mut out: Vec<RoadEnds> = Vec::new();
    let mut add = |a: NoteId, b: NoteId, route: NoteId, kind: StepKind| {
        if a == b {
            return;
        }
        let (a, b) = (a.min(b), a.max(b));
        let at = out
            .iter()
            .position(|r| (r.0, r.1) == (a, b))
            .unwrap_or_else(|| {
                out.push((a, b, Vec::new()));
                out.len() - 1
            });
        let road = &mut out[at];
        match road.2.iter_mut().find(|(r, _)| *r == route) {
            // A route taking a road twice takes it as its main road if either is.
            Some((_, was)) if kind == StepKind::Core => *was = kind,
            Some(_) => {}
            None => road.2.push((route, kind)),
        }
    };
    for path in &vault.paths {
        let Some(&first) = path.steps.first() else {
            continue;
        };
        let start = if path.quest == Quest::Main {
            Some(hub)
        } else if anchors.contains(&first) {
            None
        } else {
            let at = place(first).unwrap_or_default();
            anchors
                .iter()
                .copied()
                .filter(|&a| a != first)
                .min_by(|&a, &b| {
                    let da = place(a).unwrap_or_default().distance(at);
                    let db = place(b).unwrap_or_default().distance(at);
                    da.total_cmp(&db)
                })
        };
        if let Some(start) = start {
            add(start, first, path.note, StepKind::Core);
        }
        for (from, to, kind) in path.edges() {
            add(path.steps[from], path.steps[to], path.note, kind);
        }
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
        let name = &data.notes[node.note].name;
        let label = step.map_or_else(|| name.clone(), |step| format!("{}. {name}", step + 1));
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
    state.refold = true;
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
                "D" => Vec2::new(0.0, 300.0),
                _ => Vec2::ZERO,
            })
        };
        let roads = roads(&vault, &place, id("Hub"));
        let road = |a: &str, b: &str| {
            let (a, b) = (id(a).min(id(b)), id(a).max(id(b)));
            roads
                .iter()
                .find(|r| (r.0, r.1) == (a, b))
                .map(|r| r.2.clone())
        };
        let (main, side) = (id("Main"), id("Side"));
        assert_eq!(road("Hub", "A"), Some(vec![(main, StepKind::Core)]));
        assert_eq!(road("A", "B"), Some(vec![(main, StepKind::Core)]));
        assert_eq!(road("B", "C"), Some(vec![(main, StepKind::Optional)]));
        // The side route starts at B, the main stop nearest its first step.
        assert_eq!(road("B", "D"), Some(vec![(side, StepKind::Core)]));
        assert_eq!(road("D", "C"), Some(vec![(side, StepKind::Extra)]));
        assert_eq!(road("Hub", "D"), None);
        assert_eq!(shown_routes(&vault, None, None), vec![main]);
        assert_eq!(shown_routes(&vault, Some(&[side]), None), vec![main, side]);
    }
}
