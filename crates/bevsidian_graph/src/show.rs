//! Which notes are on the map: the hub, the steps of the routes shown and the note being
//! read. With the whole map shown, or while the map is edited, every note is.

use bevy::prelude::*;

use bevsidian_core::nav::Nav;

use crate::{GraphNode, GraphState};

/// Whether a note shows: every note does with the whole map; otherwise the ones that must
/// (`forced`: the hub and the steps of the routes shown) and the note being read.
pub fn shows(all: bool, forced: bool, current: bool) -> bool {
    all || forced || current
}

/// Shows and hides the notes; one that appears slides out to its place, from the hub's side.
pub(crate) fn show_nodes(
    nav: Res<Nav>,
    mut state: ResMut<GraphState>,
    mut nodes: Query<(&mut GraphNode, &mut Transform, &mut Visibility)>,
) {
    let all = state.whole || nav.map_edit.get_untracked();
    let current = nav.current.get_untracked();
    let mut appeared = false;
    for (mut node, mut transform, mut visibility) in &mut nodes {
        let visible = shows(all, node.forced, node.note == current);
        if visible && node.hidden {
            node.hidden = false;
            transform.translation.x = node.home.x * START;
            transform.translation.y = node.home.y * START;
            node.arriving = true;
            appeared = true;
        } else if !visible && !node.hidden {
            node.hidden = true;
        }
        visibility.set_if_neq(if node.hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    if appeared {
        state.moving = true;
    }
}

/// Where a note that appears starts, as a share of the way from the hub to its place.
const START: f32 = 0.85;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_whole_map_shows_everything_and_routes_their_steps() {
        assert!(shows(true, false, false));
        assert!(shows(false, true, false), "a step of a route shown");
        assert!(shows(false, false, true), "the note being read");
        assert!(!shows(false, false, false));
    }
}
