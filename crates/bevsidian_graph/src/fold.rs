//! Folding the map: what is not on a route the map shows waits, folded, in the note it hangs
//! from. With the whole map shown, notes beyond the concepts (sub-concepts and crates) fold
//! the same way, so the map opens on the hub, the areas and their concepts.
//!
//! A note holding folded notes shows how many with a "+N" badge. A click on it unfolds them,
//! and they float out of it; a click on an unfolded note (with a "−" badge) folds them back
//! in. A double click opens the note. The note being read is always unfolded into view, and
//! while the map is edited every note shows.

use bevy::{camera::visibility::RenderLayers, prelude::*, text::FontWeight};

use bevsidian_core::nav::Nav;
use bevsidian_ui::color;

use crate::{GraphNode, GraphState, GraphText, LAYER};

/// The deepest rank shown unfolded with the whole map: the hub (0), the areas (1) and their
/// concepts (2).
pub const SHOWN_RANK: usize = 2;

/// Radius of the badge on a note holding folded notes (world units).
pub const BADGE_RADIUS: f32 = 11.0;
/// Where the badge sits: up and right of the note's centre, as a share of its radius on each
/// axis (on its outline, at 45°).
const BADGE_AT: f32 = 0.72;
/// Size of the badge's text (world units).
const BADGE_TEXT: f32 = 12.0;
/// How close (world units) a folding note gets to its parent before it disappears.
const TUCKED: f32 = 6.0;

/// Which notes show. A note shows when it must (`forced`: the hub, a step of a route shown),
/// or when the note it hangs from shows and either lets it show (`base`) or is unfolded.
pub fn shown(parent: &[Option<usize>], base: &[bool], forced: &[bool], open: &[bool]) -> Vec<bool> {
    fn visit(
        n: usize,
        input: (&[Option<usize>], &[bool], &[bool], &[bool]),
        memo: &mut [Option<bool>],
        depth: usize,
    ) -> bool {
        if let Some(known) = memo[n] {
            return known;
        }
        let (parent, base, forced, open) = input;
        let result = forced[n]
            || match parent[n] {
                // A cycle cannot happen in a tree; stop anyway rather than recurse forever.
                Some(_) if depth > parent.len() => true,
                Some(p) => visit(p, input, memo, depth + 1) && (base[n] || open[p]),
                None => true,
            };
        memo[n] = Some(result);
        result
    }
    let mut memo = vec![None; parent.len()];
    (0..parent.len())
        .map(|n| visit(n, (parent, base, forced, open), &mut memo, 0))
        .collect()
}

/// How many notes each note holds folded: the children that show only when it is unfolded,
/// with everything that hangs from them (but not what must show anyway).
pub fn folded_counts(parent: &[Option<usize>], base: &[bool], forced: &[bool]) -> Vec<usize> {
    let count = parent.len();
    // Each note's subtree, without the notes that show anyway: walk up from every note.
    let mut size = vec![0_usize; count];
    for n in 0..count {
        if forced[n] {
            continue;
        }
        size[n] += 1;
        let mut at = parent[n];
        let mut steps = 0;
        // Up to the first note that shows anyway: it holds this one, not its ancestors.
        while let Some(p) = at {
            size[p] += 1;
            at = parent[p];
            steps += 1;
            if forced[p] || steps > count {
                break;
            }
        }
    }
    let mut folded = vec![0; count];
    for n in 0..count {
        if let Some(p) = parent[n]
            && !base[n]
            && !forced[n]
        {
            folded[p] += size[n];
        }
    }
    folded
}

/// The badge on a note holding folded notes: "+N" while they are folded, "−" while shown.
#[derive(Component)]
pub(crate) struct GraphBadge;

/// The badge's text.
#[derive(Component)]
pub(crate) struct GraphBadgeLabel;

/// The badge's disc and its outline (their materials say folded or not).
#[derive(Component)]
pub(crate) struct GraphBadgeFill;

#[derive(Component)]
pub(crate) struct GraphBadgeOutline;

/// Where a note's badge sits, relative to its centre.
pub(crate) fn badge_offset(radius: f32) -> Vec2 {
    Vec2::splat(radius * BADGE_AT)
}

/// Spawns the (hidden) badge of a note other notes hang from.
pub(crate) fn spawn_badge(node: &mut ChildSpawnerCommands, radius: f32, state: &GraphState) {
    let paint = &state.paint;
    let font = TextFont {
        font: bevy::text::FontSource::families("Geist, sans-serif"),
        font_size: BADGE_TEXT.into(),
        weight: FontWeight::SEMIBOLD,
        ..default()
    };
    node.spawn((
        GraphBadge,
        Transform::from_translation(badge_offset(radius).extend(0.3)),
        Visibility::Hidden,
        RenderLayers::layer(LAYER),
    ))
    .with_children(|badge| {
        // Outline, then the disc, then the text.
        badge.spawn((
            GraphBadgeOutline,
            Mesh2d(state.circle.clone()),
            MeshMaterial2d(paint.ring.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(BADGE_RADIUS + 1.5)),
            RenderLayers::layer(LAYER),
        ));
        badge.spawn((
            GraphBadgeFill,
            Mesh2d(state.circle.clone()),
            MeshMaterial2d(paint.badge_folded.clone()),
            Transform::from_xyz(0.0, 0.0, 0.01).with_scale(Vec3::splat(BADGE_RADIUS)),
            RenderLayers::layer(LAYER),
        ));
        badge.spawn((
            GraphBadgeLabel,
            GraphText { size: BADGE_TEXT },
            Text2d::new(""),
            font,
            bevy::text::LineHeight::RelativeToFont(1.0),
            TextColor(color::PRIMARY_FOREGROUND),
            Transform::from_xyz(0.0, 0.0, 0.02),
            RenderLayers::layer(LAYER),
        ));
    });
}

/// Applies folding: which notes show, the badges, and the motion of notes unfolding out of
/// their parent or folding back into it.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn fold_nodes(
    nav: Res<Nav>,
    mut state: ResMut<GraphState>,
    mut nodes: Query<(Entity, &mut GraphNode, &mut Transform, &mut Visibility)>,
    mut badges: Query<(Entity, &ChildOf, &mut Visibility), (With<GraphBadge>, Without<GraphNode>)>,
    mut fills: Query<
        (
            &ChildOf,
            &mut MeshMaterial2d<ColorMaterial>,
            Has<GraphBadgeOutline>,
        ),
        Or<(With<GraphBadgeFill>, With<GraphBadgeOutline>)>,
    >,
    mut labels: Query<(&ChildOf, &mut Text2d, &mut TextColor), With<GraphBadgeLabel>>,
    mut last_current: Local<Option<(usize, usize)>>,
    // Nodes already placed: a new one (the map was just built) starts folded, not folding.
    mut seen: Local<bevy::platform::collections::HashSet<Entity>>,
) {
    let entities: Vec<Entity> = nodes.iter().map(|(e, ..)| e).collect();
    if entities.is_empty() {
        return;
    }
    let index: bevy::platform::collections::HashMap<Entity, usize> =
        entities.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    let parent: Vec<Option<usize>> = nodes
        .iter()
        .map(|(_, n, ..)| n.parent.and_then(|p| index.get(&p).copied()))
        .collect();
    let editing = nav.map_edit.get_untracked();
    let whole = state.whole || editing;
    let base: Vec<bool> = nodes
        .iter()
        .map(|(_, n, ..)| editing || (whole && n.rank <= SHOWN_RANK))
        .collect();
    let forced: Vec<bool> = nodes.iter().map(|(_, n, ..)| editing || n.forced).collect();
    let mut open: Vec<bool> = nodes.iter().map(|(_, n, ..)| n.open).collect();

    // Requests: a click on a note or its badge, "expand all" / "collapse all", and the note
    // being read (unfolded into view when it changes, or when the map is rebuilt).
    let mut changed = std::mem::take(&mut state.refold);
    if let Some(toggled) = state
        .fold_toggle
        .take()
        .and_then(|e| index.get(&e).copied())
    {
        open[toggled] = !open[toggled];
        changed = true;
    }
    if let Some(all) = state.fold_all.take() {
        open.fill(all);
        changed = true;
    }
    let current = nav.current.get_untracked();
    if *last_current != Some((current, entities.len())) {
        *last_current = Some((current, entities.len()));
        if let Some(mut at) = nodes.iter().position(|(_, n, ..)| n.note == current) {
            while let Some(p) = parent[at] {
                changed |= !open[p];
                open[p] = true;
                at = p;
            }
        }
    }
    let visible = shown(&parent, &base, &forced, &open);
    let folds = folded_counts(&parent, &base, &forced);
    let positions: Vec<Vec2> = nodes
        .iter()
        .map(|(_, _, t, _)| t.translation.truncate())
        .collect();
    let mut moving = false;
    seen.retain(|e| index.contains_key(e));
    for (i, (entity, mut node, mut transform, mut visibility)) in nodes.iter_mut().enumerate() {
        let new = seen.insert(entity);
        node.open = open[i];
        node.folds = if editing { 0 } else { folds[i] };
        let parent_at = parent[i].map(|p| positions[p]);
        if visible[i] {
            if node.hidden
                && !new
                && let Some(from) = parent_at
            {
                // Unfolding: float out of the parent to its place.
                transform.translation.x = from.x;
                transform.translation.y = from.y;
                node.arriving = true;
                moving = true;
            }
            node.hidden = false;
            node.tucking = false;
        } else if node.hidden {
            node.tucking = false;
        } else if new || node.arriving || parent_at.is_none() {
            node.hidden = true;
            node.tucking = false;
        } else if let Some(to) = parent_at {
            // Float into the parent, then disappear.
            if transform.translation.truncate().distance(to) < TUCKED {
                node.hidden = true;
                node.tucking = false;
            } else {
                node.tucking = true;
                node.tuck_to = to;
                moving = true;
            }
        }
        visibility.set_if_neq(if node.hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    if changed || moving {
        state.moving = true;
    }

    // Badges: "+N" on white while folded, "−" on the card colour while unfolded.
    let paint = &state.paint;
    let node_of = |e: Entity| nodes.get(e).ok().map(|(_, n, ..)| n);
    let mut note_of_badge = bevy::platform::collections::HashMap::<Entity, Entity>::default();
    for (badge, child_of, mut visibility) in &mut badges {
        note_of_badge.insert(badge, child_of.parent());
        // Zoomed far out (the note grew to stay visible), the badge would be a speck.
        let shows = node_of(child_of.parent())
            .is_some_and(|n| !n.hidden && !n.tucking && n.folds > 0)
            && nodes
                .get(child_of.parent())
                .is_ok_and(|(_, _, t, _)| t.scale.x <= 1.0);
        visibility.set_if_neq(if shows {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let owner = |part: &ChildOf| {
        note_of_badge
            .get(&part.parent())
            .and_then(|&note| node_of(note))
    };
    for (child_of, mut material, outline) in &mut fills {
        let Some(node) = owner(child_of) else {
            continue;
        };
        let wanted = match (outline, node.open) {
            (false, true) => &paint.badge_open,
            (false, false) => &paint.badge_folded,
            (true, true) => &paint.ring_hovered,
            (true, false) => &paint.ring,
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
    for (child_of, mut text, mut tint) in &mut labels {
        let Some(node) = owner(child_of) else {
            continue;
        };
        let (wanted, color) = if node.open {
            ("\u{2212}".to_owned(), color::FOREGROUND)
        } else {
            (format!("+{}", node.folds), color::PRIMARY_FOREGROUND)
        };
        if text.0 != wanted {
            text.0 = wanted;
        }
        tint.set_if_neq(TextColor(color));
    }
}

/// The note whose badge is under `world`, if any.
pub(crate) fn badge_at(
    world: Vec2,
    nodes: &Query<(Entity, &Transform, &GraphNode)>,
) -> Option<Entity> {
    nodes
        .iter()
        .filter(|(_, t, n)| !n.hidden && !n.tucking && n.folds > 0 && t.scale.x <= 1.0)
        .find(|(_, t, n)| {
            let at = t.translation.truncate() + badge_offset(n.radius) * t.scale.x;
            at.distance(world) <= BADGE_RADIUS * t.scale.x + 4.0
        })
        .map(|(e, ..)| e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_show_their_steps_and_the_rest_folds() {
        // 0 hub → 1 area → 2 concept → 3 sub-concept → 4 crate; 5 a crate under the area.
        let parent = [None, Some(0), Some(1), Some(2), Some(3), Some(1)];
        let closed = [false; 6];
        // Only a route through the sub-concept: the hub and it show, nothing else.
        let forced = [true, false, false, true, false, false];
        let none = [false; 6];
        assert_eq!(
            shown(&parent, &none, &forced, &closed),
            [true, false, false, true, false, false]
        );
        // The sub-concept holds its crate; the hub holds the area, its concept and its crate
        // (not the sub-concept, which shows anyway, nor its crate).
        assert_eq!(folded_counts(&parent, &none, &forced), [3, 2, 0, 1, 0, 0]);
        let mut open = closed;
        open[3] = true;
        assert!(shown(&parent, &none, &forced, &open)[4]);
    }

    #[test]
    fn the_whole_map_folds_beyond_the_concepts() {
        let parent = [None, Some(0), Some(1), Some(2), Some(3), Some(1)];
        let rank = [0, 1, 2, 3, 4, 4];
        let base: Vec<bool> = rank.iter().map(|&r| r <= SHOWN_RANK).collect();
        let forced = [true, false, false, false, false, false];
        let closed = [false; 6];
        assert_eq!(
            shown(&parent, &base, &forced, &closed),
            [true, true, true, false, false, false]
        );
        assert_eq!(folded_counts(&parent, &base, &forced), [0, 1, 2, 1, 0, 0]);
        // Unfolding the concept shows its sub-concept, whose crate stays folded in it.
        let mut open = closed;
        open[2] = true;
        assert_eq!(
            shown(&parent, &base, &forced, &open),
            [true, true, true, true, false, false]
        );
        open[3] = true;
        assert!(shown(&parent, &base, &forced, &open)[4]);
        // Folding the concept again hides everything under it, unfolded or not.
        open[2] = false;
        assert_eq!(
            shown(&parent, &base, &forced, &open),
            [true, true, true, false, false, false]
        );
    }
}
