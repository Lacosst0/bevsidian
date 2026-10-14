//! Folding the graph: notes beyond the concepts' ring start folded into the note they grow
//! from, so the whole graph opens on the hub, the areas and their concepts.
//!
//! A note holding folded notes shows how many with a "+N" badge and two rings around its
//! disc; clicking it (or the badge) unfolds them, and they float out of it. An unfolded
//! note shows a "−" badge, which folds them back in. The note being read is always unfolded
//! into view, and a followed path shows its steps whatever is folded.

use bevy::{camera::visibility::RenderLayers, prelude::*, text::FontWeight};

use bevsidian_core::nav::Nav;
use bevsidian_ui::color;

use crate::{GraphNode, GraphState, GraphText, LAYER};

/// The outermost ring shown unfolded: the hub (0), the areas (1) and their concepts (2).
pub const SHOWN_RING: usize = 2;

/// Radius of the badge on a note holding folded notes (world units).
pub const BADGE_RADIUS: f32 = 11.0;
/// Where the badge sits: up and right of the disc's centre, as a share of its radius on
/// each axis (on the outline, at 45°).
const BADGE_AT: f32 = 0.72;
/// Size of the badge's text (world units).
const BADGE_TEXT: f32 = 12.0;
/// The rings around a note holding folded notes: how far past its disc each starts, and how
/// wide they are (world units).
const RINGS: [f32; 2] = [4.0, 9.0];
const RING_WIDTH: f32 = 1.5;
/// How far (world units) the app's tree may be from its wanted distance before it moves:
/// notes settling nudge the course's reach a little all the time.
const SHIFT_SLACK: f32 = 120.0;
/// How close (world units) a folding note gets to its parent before it disappears.
const TUCKED: f32 = 6.0;

/// Which notes are shown: a note is shown when the note it grows from is, and either sits
/// within [`SHOWN_RING`] or grows from an unfolded (`open`) note. Roots are always shown.
pub fn shown(parent: &[Option<usize>], ring: &[usize], open: &[bool]) -> Vec<bool> {
    fn visit(
        n: usize,
        parent: &[Option<usize>],
        ring: &[usize],
        open: &[bool],
        memo: &mut [Option<bool>],
        depth: usize,
    ) -> bool {
        if let Some(known) = memo[n] {
            return known;
        }
        let result = match parent[n] {
            // A cycle cannot happen in a tree; stop anyway rather than recurse forever.
            Some(_) if depth > parent.len() => true,
            Some(p) => {
                visit(p, parent, ring, open, memo, depth + 1) && (ring[n] <= SHOWN_RING || open[p])
            }
            None => true,
        };
        memo[n] = Some(result);
        result
    }
    let mut memo = vec![None; parent.len()];
    (0..parent.len())
        .map(|n| visit(n, parent, ring, open, &mut memo, 0))
        .collect()
}

/// How many notes each note holds folded: its children beyond [`SHOWN_RING`] with everything
/// growing from them. A note with none has no badge.
pub fn folded_counts(parent: &[Option<usize>], ring: &[usize]) -> Vec<usize> {
    let count = parent.len();
    // Each note's subtree size: walk up from every note, counting it at each ancestor.
    let mut size = vec![1_usize; count];
    for n in 0..count {
        let mut at = parent[n];
        let mut steps = 0;
        while let Some(p) = at {
            size[p] += 1;
            at = parent[p];
            steps += 1;
            if steps > count {
                break;
            }
        }
    }
    let mut folded = vec![0; count];
    for n in 0..count {
        if let Some(p) = parent[n]
            && ring[n] > SHOWN_RING
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

/// The rings around a note while it holds folded notes.
#[derive(Component)]
pub(crate) struct GraphStack;

/// Where a note's badge sits, relative to its centre.
pub(crate) fn badge_offset(radius: f32) -> Vec2 {
    Vec2::splat(radius * BADGE_AT)
}

/// Spawns the badge and the rings of a note that can hold folded notes.
pub(crate) fn spawn_badge(
    node: &mut ChildSpawnerCommands,
    radius: f32,
    state: &GraphState,
    meshes: &mut Assets<Mesh>,
) {
    let paint = &state.paint;
    for (i, gap) in RINGS.iter().enumerate() {
        let inner = radius + gap;
        node.spawn((
            GraphStack,
            Mesh2d(meshes.add(Annulus::new(inner, inner + RING_WIDTH))),
            // The inner ring brighter: ripples around a note with more inside.
            MeshMaterial2d(paint.stack[i].clone()),
            Transform::from_xyz(0.0, 0.0, -0.3),
            Visibility::Hidden,
            RenderLayers::layer(LAYER),
        ));
    }
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

/// Applies folding: which notes are shown, the badges, and the motion of notes unfolding
/// out of their parent or folding back into it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fold_nodes(
    nav: Res<Nav>,
    mut state: ResMut<GraphState>,
    mut nodes: Query<(Entity, &mut GraphNode, &mut Transform, &mut Visibility)>,
    mut badges: Query<
        (Entity, &ChildOf, &mut Visibility),
        (With<GraphBadge>, Without<GraphNode>, Without<GraphStack>),
    >,
    mut stacks: Query<
        (&ChildOf, &mut Visibility),
        (With<GraphStack>, Without<GraphNode>, Without<GraphBadge>),
    >,
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
    // Nodes already placed: a new one (the graph was just built) starts folded, not folding.
    mut seen: Local<bevy::platform::collections::HashSet<Entity>>,
) {
    let entities: Vec<Entity> = nodes.iter().map(|(e, ..)| e).collect();
    if entities.is_empty() {
        return;
    }
    let index = |e: Entity| entities.iter().position(|&x| x == e);
    let parent: Vec<Option<usize>> = nodes
        .iter()
        .map(|(_, n, ..)| n.parent.and_then(index))
        .collect();
    let ring: Vec<usize> = nodes.iter().map(|(_, n, ..)| n.ring).collect();
    let mut open: Vec<bool> = nodes.iter().map(|(_, n, ..)| n.open).collect();

    // Requests: a click on a note or its badge, "expand all" / "collapse all", and the note
    // being read (unfolded into view when it changes, or when the graph is rebuilt).
    let mut changed = false;
    if let Some(toggled) = state.fold_toggle.take().and_then(index) {
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
    let visible = shown(&parent, &ring, &open);
    let positions: Vec<Vec2> = nodes
        .iter()
        .map(|(_, _, t, _)| t.translation.truncate())
        .collect();
    let following = !state.path.is_empty();
    let mut moving = false;
    seen.retain(|e| entities.contains(e));
    for (i, (entity, mut node, mut transform, mut visibility)) in nodes.iter_mut().enumerate() {
        let new = seen.insert(entity);
        node.open = open[i];
        let hide = if following {
            node.off_path
        } else {
            !visible[i]
        };
        let parent_at = parent[i].map(|p| positions[p]);
        if hide {
            if node.hidden {
                node.tucking = false;
            } else if new || following || node.arriving || parent_at.is_none() {
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
        } else {
            if node.hidden
                && !following
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
        }
        visibility.set_if_neq(if node.hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    // The app's tree keeps its distance from the course's as far as that is unfolded: it
    // floats closer when notes fold away and further when they unfold.
    let reach = nodes
        .iter()
        .filter(|(_, n, ..)| !n.app && !n.hidden)
        .map(|(_, n, ..)| n.home.length() + n.orbit)
        .fold(0.0, f32::max);
    let wanted = reach + state.app_reach + crate::APP_GAP;
    if !following && reach > 0.0 && (wanted - state.app_shift).abs() > SHIFT_SLACK {
        let by = wanted - state.app_shift;
        state.app_shift = wanted;
        for (_, mut node, ..) in &mut nodes {
            if node.app {
                node.home.x += by;
                node.arriving = true;
            }
        }
        moving = true;
    }
    if changed || moving {
        state.alpha = state.alpha.max(0.3);
    }

    // Badges: "+N" on white while folded, "−" on the card colour while unfolded.
    let paint = &state.paint;
    let badge_of = |child_of: &ChildOf| nodes.get(child_of.parent()).ok();
    // The badge's parts hang from the badge, which hangs from the note.
    let mut note_of_badge = bevy::platform::collections::HashMap::<Entity, Entity>::default();
    for (badge, child_of, mut visibility) in &mut badges {
        note_of_badge.insert(badge, child_of.parent());
        let shows = badge_of(child_of)
            .is_some_and(|(_, n, ..)| !n.hidden && !n.tucking && !following && n.folds > 0);
        visibility.set_if_neq(if shows {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (child_of, mut visibility) in &mut stacks {
        let shows = badge_of(child_of).is_some_and(|(_, n, ..)| {
            !n.hidden && !n.tucking && !following && n.folds > 0 && !n.open
        });
        visibility.set_if_neq(if shows {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let node_of = |part: &ChildOf| {
        note_of_badge
            .get(&part.parent())
            .and_then(|&note| nodes.get(note).ok())
    };
    for (child_of, mut material, outline) in &mut fills {
        let Some((_, node, ..)) = node_of(child_of) else {
            continue;
        };
        // Folded: a white disc. Unfolded: a dark one with a light outline.
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
    for (child_of, mut text, mut color) in &mut labels {
        let Some((_, node, ..)) = node_of(child_of) else {
            continue;
        };
        let (wanted, tint) = if node.open {
            ("\u{2212}".to_owned(), color::FOREGROUND)
        } else {
            (format!("+{}", node.folds), color::PRIMARY_FOREGROUND)
        };
        if text.0 != wanted {
            text.0 = wanted;
        }
        color.set_if_neq(TextColor(tint));
    }
}

/// The note whose badge is under `world`, if any.
pub(crate) fn badge_at(
    world: Vec2,
    nodes: &Query<(Entity, &Transform, &GraphNode)>,
) -> Option<Entity> {
    nodes
        .iter()
        .filter(|(_, _, n)| !n.hidden && !n.tucking && n.folds > 0)
        .find(|(_, t, n)| {
            let at = t.translation.truncate() + badge_offset(n.radius);
            at.distance(world) <= BADGE_RADIUS + 4.0
        })
        .map(|(e, ..)| e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folded_note_unfolds_out_of_its_parent_and_folds_back_in() {
        use bevsidian_core::nav::View;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            // Every update is one 60 Hz frame.
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f32(1.0 / 60.0),
            ))
            .insert_resource(Nav::new(99))
            .insert_resource(GraphState {
                alpha: 1.0,
                ..default()
            })
            .add_systems(Update, (fold_nodes, crate::simulate).chain());
        app.world().resource::<Nav>().view.set(View::Graph);
        let node = |note: usize, ring: usize, home: Vec2, parent: Option<Entity>, folds| {
            (
                GraphNode {
                    note,
                    velocity: Vec2::ZERO,
                    radius: 20.0,
                    pinned: false,
                    neighbors: Vec::new(),
                    hidden: false,
                    off_path: false,
                    parent,
                    ring,
                    app: false,
                    folds,
                    open: false,
                    tucking: false,
                    tuck_to: Vec2::ZERO,
                    home,
                    arriving: false,
                    target: None,
                    orbit: 30.0,
                },
                Transform::from_translation(home.extend(0.0)),
                Visibility::default(),
            )
        };
        let concept = app.world_mut().spawn(node(0, 2, Vec2::ZERO, None, 1)).id();
        let library = app
            .world_mut()
            .spawn(node(1, 4, Vec2::new(300.0, 0.0), Some(concept), 0))
            .id();
        let run = |app: &mut App, frames: usize| {
            for _ in 0..frames {
                app.update();
            }
        };
        let get = |app: &App, e: Entity| {
            let node = app.world().get::<GraphNode>(e).unwrap();
            let at = app
                .world()
                .get::<Transform>(e)
                .unwrap()
                .translation
                .truncate();
            (node.hidden, node.open, at)
        };
        run(&mut app, 1);
        assert!(get(&app, library).0, "beyond the concepts: folded at first");

        app.world_mut().resource_mut::<GraphState>().fold_toggle = Some(concept);
        run(&mut app, 1);
        let (hidden, _, at) = get(&app, library);
        assert!(!hidden && at.x < 50.0, "it unfolds out of its parent: {at}");
        assert!(get(&app, concept).1, "the parent is open");
        run(&mut app, 240);
        let (_, _, at) = get(&app, library);
        assert!((at.x - 300.0).abs() < 5.0, "and floats home: {at}");

        app.world_mut().resource_mut::<GraphState>().fold_toggle = Some(concept);
        run(&mut app, 1);
        assert!(
            !get(&app, library).0,
            "folding, it first floats back into its parent"
        );
        run(&mut app, 240);
        let (hidden, _, at) = get(&app, library);
        assert!(hidden && at.x < 10.0, "then disappears there: {at}");
        // Its home is kept for the next unfold.
        let home = app.world().get::<GraphNode>(library).unwrap().home;
        assert!((home.x - 300.0).abs() < 2.0, "{home}");
    }

    #[test]
    fn notes_beyond_the_concepts_start_folded() {
        // 0 hub → 1 area → 2 concept → 3 sub-concept → 4 library; 5 a library under the area.
        let parent = [None, Some(0), Some(1), Some(2), Some(3), Some(1)];
        let ring = [0, 1, 2, 3, 4, 4];
        let closed = [false; 6];
        assert_eq!(
            shown(&parent, &ring, &closed),
            [true, true, true, false, false, false]
        );
        assert_eq!(folded_counts(&parent, &ring), [0, 1, 2, 1, 0, 0]);
        // Unfolding the concept shows its sub-concept, whose library stays folded in it.
        let mut open = closed;
        open[2] = true;
        assert_eq!(
            shown(&parent, &ring, &open),
            [true, true, true, true, false, false]
        );
        open[3] = true;
        assert!(shown(&parent, &ring, &open)[4]);
        // Folding the concept again hides everything under it, unfolded or not.
        open[2] = false;
        assert_eq!(
            shown(&parent, &ring, &open),
            [true, true, true, false, false, false]
        );
    }
}
