//! Obsidian-style graph view.
//!
//! Notes, drawn as their icons in bordered discs, are simulated as a force-directed graph in a 2D world on its own render
//! layer. A dedicated camera renders that world into an image that the UI shows in the
//! [`GraphViewport`] node; pointer events on the node are mapped back into the world for
//! hovering, clicking, dragging nodes, panning (with momentum) and zooming (wheel, pinch
//! and buttons). A press only opens a note if the pointer did not move: drags never click.
//!
//! Lines say how two notes are connected ([`Connection`]): solid when they link to each
//! other, dashed when one links to the other, dotted when they only share a tag.

use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    input::touch::{Touch, Touches},
    picking::events::{
        PointerClick, PointerDrag, PointerDragEnd, PointerMove, PointerOut, PointerPress,
        PointerRelease, PointerScroll,
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
    text::{FontWeight, LineHeight},
    ui::UiGlobalTransform,
};

use bevsidian_core::{
    nav::{Nav, View},
    vault::{NoteId, NotePath, StepKind, Vault, VaultData},
};
use bevsidian_ui::{Size, Tooltip, Variant, caption, color, icon_button, radius, shadow};

mod layout;

const LAYER: usize = 1;

/// Adds the graph view; place a [`GraphViewport`] node in the UI to show it.
pub fn plugin(app: &mut App) {
    app.init_resource::<GraphState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_nodes,
                follow_path,
                show_legend,
                attach_viewport,
                toggle_camera,
                resize_target,
                simulate,
                frame_path,
                pinch_zoom,
                camera_motion,
                sharpen_labels,
                style_nodes,
                draw_edges,
            )
                .chain(),
        );
}

/// The mesh holding every edge as a thin quad (cheaper than gizmos, which add dozens of
/// systems that run every frame).
#[derive(Component)]
struct EdgeMesh;

/// UI node that displays the graph.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct GraphViewport;

#[derive(Component)]
struct GraphCamera;

/// A note in the graph, drawn as its icon with its name below.
#[derive(Component)]
struct GraphNode {
    note: NoteId,
    velocity: Vec2,
    radius: f32,
    /// The hub: it stays in the middle.
    pinned: bool,
    neighbors: Vec<Entity>,
    /// Off the reading path being followed: not drawn, simulated or picked.
    hidden: bool,
    /// Its place in the whole graph: first from [`layout::clusters`], then wherever it was
    /// last pushed or dropped. It floats there when the graph opens or a path is left.
    home: Vec2,
    /// Floating to its home (the graph just opened, or a path was left).
    arriving: bool,
    /// Its place on the followed path's line (see [`layout::line`]).
    target: Option<Vec2>,
    /// Radius of its orbit: no other note comes closer (room for its disc and label).
    orbit: f32,
}

#[derive(Component)]
struct GraphLabel;

#[derive(Component)]
struct GraphIcon;

/// The disc behind a node's icon.
#[derive(Component)]
struct GraphFill;

/// The outline around a node's disc.
#[derive(Component)]
struct GraphRing;

/// Text drawn in the graph world, re-rasterized as the camera zooms so it stays sharp.
#[derive(Component)]
struct GraphText {
    /// Size in world units.
    size: f32,
}

#[derive(Resource, Default)]
pub struct GraphState {
    image: Handle<Image>,
    /// Links, one per direction.
    edges: Vec<(Entity, Entity)>,
    /// The springs of the simulation: a link and its length in the layout.
    springs: Vec<(Entity, Entity, f32)>,
    /// The lines drawn, one per connected pair, and whether the whole graph shows it: only
    /// lines inside a cluster and from the hub to cluster centres do, the others only for
    /// the hovered note (they would tangle the clusters together).
    lines: Vec<(Entity, Entity, Connection, bool)>,
    /// The steps of the reading path being followed, in order (empty when none is).
    path: Vec<Entity>,
    /// The lines of that path: core steps in a chain, optional and extra ones hanging off.
    path_lines: Vec<(Entity, Entity, Connection)>,
    /// The camera frames the shown notes while they settle (after a path is chosen), until
    /// the user moves it.
    framing: bool,
    hovered: Option<Entity>,
    dragging: Option<Entity>,
    /// The current press, until the next one.
    press: Option<Press>,
    /// Panning momentum after a fling, in world units per second.
    pan_velocity: Vec2,
    /// Camera position and zoom being eased towards (zoom buttons, "Fit").
    camera_goal: Option<(Vec2, f32)>,
    /// Distance between two fingers in the previous frame of a pinch.
    pinch: Option<f32>,
    /// Simulation "temperature": decays to zero, re-heated by interaction.
    alpha: f32,
    /// The node last dragged: while the graph settles, nodes far from it barely move.
    anchor: Option<Entity>,
    /// Shared node mesh and materials.
    circle: Handle<Mesh>,
    paint: NodePaint,
}

/// Materials of a node's disc and outline, per state.
#[derive(Default)]
struct NodePaint {
    fill: Handle<ColorMaterial>,
    ring: Handle<ColorMaterial>,
    ring_hovered: Handle<ColorMaterial>,
    current_fill: Handle<ColorMaterial>,
    current_ring: Handle<ColorMaterial>,
}

impl GraphState {
    /// Whether the simulation or a drag still needs frames.
    pub fn animating(&self) -> bool {
        self.alpha >= 0.002
            || self.dragging.is_some()
            || self.pan_velocity != Vec2::ZERO
            || self.camera_goal.is_some()
            || self.pinch.is_some()
    }
}

/// A primary-button press on the graph.
#[derive(Clone, Copy, Debug)]
struct Press {
    /// Node under the pointer when pressed.
    node: Option<Entity>,
    /// Whether the pointer moved far enough to count as a drag (then it never clicks).
    moved: bool,
}

/// How far (logical px) a press may move and still count as a click.
const CLICK_SLOP: f32 = 6.0;
/// The same for fingers, which wobble more.
const TOUCH_CLICK_SLOP: f32 = 12.0;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<GraphState>,
) {
    commands.spawn((
        EdgeMesh,
        Mesh2d(meshes.add(Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        ))),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        Transform::default(),
        RenderLayers::layer(LAYER),
    ));

    let image = Image::new_target_texture(
        512,
        512,
        // A plain sRGB format: WebGL2 does not support separate view formats.
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    state.image = images.add(image);

    commands.spawn((
        GraphCamera,
        Camera2d,
        Camera {
            order: -1,
            is_active: false,
            clear_color: ClearColorConfig::Custom(color::BACKGROUND),
            ..default()
        },
        RenderTarget::Image(state.image.clone().into()),
        RenderLayers::layer(LAYER),
        Msaa::Sample4,
    ));

    // An opaque disc behind each icon, so edges stop at its outline.
    state.paint = NodePaint {
        fill: materials.add(color::CARD),
        ring: materials.add(RING_COLOR),
        ring_hovered: materials.add(color::MUTED_FOREGROUND),
        current_fill: materials.add(color::LINK),
        current_ring: materials.add(color::LINK_RING),
    };
    state.circle = meshes.add(Circle::new(1.0));
}

/// (Re)builds the nodes and edges from the vault, at startup and after a note is edited.
/// Notes keep their place.
fn spawn_nodes(
    mut commands: Commands,
    vault: Res<Vault>,
    mut state: ResMut<GraphState>,
    old: Query<(Entity, &GraphNode, &Transform)>,
    mut built: Local<Option<u64>>,
) {
    let vault = vault.get_untracked();
    if *built == Some(vault.revision) {
        return;
    }
    *built = Some(vault.revision);
    let mut kept = std::collections::HashMap::new();
    for (entity, node, transform) in &old {
        kept.insert(node.note, transform.translation);
        commands.entity(entity).despawn();
    }
    state.hovered = None;
    state.dragging = None;
    state.press = None;
    let (fill, ring, circle) = (
        state.paint.fill.clone(),
        state.paint.ring.clone(),
        state.circle.clone(),
    );
    let font = |family: &'static str, size: f32, weight: FontWeight| TextFont {
        font: bevy::text::FontSource::families(family),
        font_size: size.into(),
        weight,
        ..default()
    };

    // Path notes stay out of the graph: each would link to every note on its path (paths
    // show as filters instead).
    let shown: Vec<NoteId> = (0..vault.notes.len())
        .filter(|&i| vault.path(i).is_none())
        .collect();
    let layout = home_positions(&vault, &shown);
    let note_count = vault.notes.len();
    let mut note_entities = Vec::with_capacity(note_count);
    for (i, note) in vault.notes.iter().enumerate() {
        let Some(slot) = shown.iter().position(|&n| n == i) else {
            note_entities.push(None);
            continue;
        };
        let home = layout.homes[slot];
        let degree = note.outgoing.len()
            + note
                .backlinks
                .iter()
                .filter(|&&b| vault.path(b).is_none())
                .count();
        // Better-linked notes are bigger.
        let radius = 16.0 + 4.0 * (degree as f32).sqrt();
        // The hub (`hub: true`) is the centre everything radiates from: bigger still.
        let hub = note
            .doc
            .properties
            .iter()
            .any(|(key, value)| key == "hub" && value == "true");
        let radius = if hub { radius * 1.6 } else { radius };
        let icon_size = (radius * 0.9).round();
        let id = commands
            .spawn((
                GraphNode {
                    note: i,
                    velocity: Vec2::ZERO,
                    radius,
                    pinned: hub,
                    neighbors: Vec::new(),
                    hidden: false,
                    home,
                    arriving: kept.is_empty(),
                    target: None,
                    orbit: orbit(radius, &note.name),
                },
                // Notes float out from the hub to their homes when the graph is first built.
                Transform::from_translation(kept.get(&i).copied().unwrap_or_else(|| {
                    let start = if kept.is_empty() {
                        home * START_SPREAD
                    } else {
                        home
                    };
                    start.extend(1.0)
                })),
                Visibility::default(),
                RenderLayers::layer(LAYER),
            ))
            .with_children(|node| {
                node.spawn((
                    GraphRing,
                    Mesh2d(circle.clone()),
                    MeshMaterial2d(ring.clone()),
                    Transform::from_xyz(0.0, 0.0, -0.2).with_scale(Vec3::splat(radius)),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphFill,
                    Mesh2d(circle.clone()),
                    MeshMaterial2d(fill.clone()),
                    Transform::from_xyz(0.0, 0.0, -0.1)
                        .with_scale(Vec3::splat(radius - RING_WIDTH)),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphIcon,
                    GraphText { size: icon_size },
                    Text2d::new(note.icon.to_string()),
                    font("lucide", icon_size, FontWeight::NORMAL),
                    // One-em line: Lucide icons are centred in their em box.
                    LineHeight::RelativeToFont(1.0),
                    TextColor(NOTE_COLOR),
                    Transform::default(),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphLabel,
                    GraphText { size: LABEL_SIZE },
                    Text2d::new(note.name.clone()),
                    font("Geist, sans-serif", LABEL_SIZE, FontWeight::NORMAL),
                    TextColor(color::MUTED_FOREGROUND),
                    Transform::from_xyz(0.0, -radius - 12.0, 0.0),
                    RenderLayers::layer(LAYER),
                ));
            })
            .id();
        note_entities.push(Some(id));
    }
    let slot = |id: NoteId| shown.iter().position(|&n| n == id);
    let mut edges = Vec::new();
    let mut springs = Vec::new();
    for (i, note) in vault.notes.iter().enumerate() {
        for &target in &note.outgoing {
            if let (Some(a), Some(b), Some(sa), Some(sb)) = (
                note_entities[i],
                note_entities[target],
                slot(i),
                slot(target),
            ) {
                edges.push((a, b));
                springs.push((a, b, layout.homes[sa].distance(layout.homes[sb])));
            }
        }
    }
    state.edges = edges;
    state.springs = springs;
    state.lines = connections(&vault)
        .into_iter()
        .filter_map(|(a, b, kind)| {
            let (sa, sb) = (slot(a)?, slot(b)?);
            // Inside a cluster, and the spokes from the hub to each cluster's centre.
            let spoke = |hub: usize, centre: usize| hub == layout.root && layout.centres[centre];
            let overview = layout.groups[sa] == layout.groups[sb] || spoke(sa, sb) || spoke(sb, sa);
            Some((note_entities[a]?, note_entities[b]?, kind, overview))
        })
        .collect();
    state.anchor = None;
    if kept.is_empty() {
        state.alpha = 1.0;
        state.framing = true;
    } else {
        // An edit only adds or removes a few links: settle gently.
        state.alpha = state.alpha.max(0.1);
    }
}

/// Where notes start when the graph is first built, as a share of the way to their homes.
const START_SPREAD: f32 = 0.1;

/// Radius a note keeps clear around itself: its disc, and its label (centred below it).
fn orbit(radius: f32, name: &str) -> f32 {
    let label = name.chars().count() as f32 * LABEL_SIZE * 0.55;
    (radius + ORBIT_MARGIN).max(label / 2.0 + ORBIT_MARGIN / 2.0)
}

/// Room around a note's disc that others keep out of.
const ORBIT_MARGIN: f32 = 22.0;

/// Where the shown notes go in the whole graph.
struct Layout {
    /// Home of each shown note, in the order given.
    homes: Vec<Vec2>,
    /// Its cluster (its top folder).
    groups: Vec<usize>,
    /// Whether it is its cluster's centre.
    centres: Vec<bool>,
    /// The hub's place in the order given.
    root: usize,
}

/// Home positions of the `shown` notes (in that order): a cluster per top folder around the
/// hub's (`hub: true`, or else the best-linked note).
fn home_positions(vault: &VaultData, shown: &[NoteId]) -> Layout {
    let slot = |id: NoteId| shown.iter().position(|&n| n == id);
    let mut links = Vec::new();
    for (a, &id) in shown.iter().enumerate() {
        for &target in &vault.notes[id].outgoing {
            if let Some(b) = slot(target) {
                links.push((a, b));
            }
        }
    }
    let hub = shown.iter().position(|&id| {
        vault.notes[id]
            .doc
            .properties
            .iter()
            .any(|(key, value)| key == "hub" && value == "true")
    });
    let root = hub.unwrap_or_else(|| {
        (0..shown.len())
            .max_by_key(|&a| links.iter().filter(|&&(x, y)| x == a || y == a).count())
            .unwrap_or(0)
    });
    let mut folders: Vec<&str> = Vec::new();
    let groups: Vec<usize> = shown
        .iter()
        .map(|&id| {
            let folder = vault.notes[id].folder.split('/').next().unwrap_or("");
            folders
                .iter()
                .position(|&f| f == folder)
                .unwrap_or_else(|| {
                    folders.push(folder);
                    folders.len() - 1
                })
        })
        .collect();
    let space: Vec<f32> = shown
        .iter()
        .map(|&id| 2.0 * orbit(20.0, &vault.notes[id].name))
        .collect();
    let homes = layout::clusters(&groups, &links, root, &space);
    let centre = layout::centres(&groups, &links);
    let centres = (0..shown.len()).map(|n| centre[groups[n]] == n).collect();
    Layout {
        homes,
        groups,
        centres,
        root,
    }
}

/// Icon colours.
const NOTE_COLOR: Color = color::BODY;
const HOVERED_COLOR: Color = color::FOREGROUND;
const DIM_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
/// The outline of a disc: the border token over the page, made opaque.
const RING_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
/// Width of the outline, in world units.
const RING_WIDTH: f32 = 1.5;
/// How far the halo of the current note reaches past its disc.
const HALO: f32 = 5.0;

/// Shows the render target in a freshly spawned viewport node and wires pointer input.
fn attach_viewport(
    mut commands: Commands,
    added: Query<Entity, Added<GraphViewport>>,
    state: Res<GraphState>,
) {
    for entity in &added {
        commands
            .entity(entity)
            .insert(ImageNode::new(state.image.clone()))
            .observe(on_move)
            .observe(on_out)
            .observe(on_press)
            .observe(on_release)
            .observe(on_click)
            .observe(on_drag)
            .observe(on_drag_end)
            .observe(on_scroll);
    }
}

fn toggle_camera(nav: Res<Nav>, mut cameras: Query<&mut Camera, With<GraphCamera>>) {
    let active = nav.view.get_untracked() == View::Graph;
    for mut camera in &mut cameras {
        if camera.is_active != active {
            camera.is_active = active;
        }
    }
}

/// Keeps the render target the same size as the viewport node.
fn resize_target(
    viewports: Query<&ComputedNode, With<GraphViewport>>,
    state: Res<GraphState>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(node) = viewports.single() else {
        return;
    };
    let size = node.size.max(Vec2::ONE).as_uvec2();
    let Some(image) = images.get(&state.image) else {
        return;
    };
    if image.size() != size
        && let Some(mut image) = images.get_mut(&state.image)
    {
        {
            image.resize(Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            });
        }
    }
}

/// Shows the notes of the followed path, numbered in order, and hides the rest (or shows
/// every note when no path is followed).
fn follow_path(
    nav: Res<Nav>,
    vault: Res<Vault>,
    mut state: ResMut<GraphState>,
    mut nodes: Query<(Entity, &mut GraphNode, &mut Visibility, &Children)>,
    mut labels: Query<&mut Text2d, With<GraphLabel>>,
    // The path shown, and the vault revision whose nodes showed it (nodes are rebuilt when
    // the vault changes).
    mut last: Local<Option<(Option<NoteId>, u64)>>,
) {
    let path = nav.path.get_untracked();
    let data = vault.get_untracked();
    if *last == Some((path, data.revision)) {
        return;
    }
    let changed = last.is_some_and(|(shown, _)| shown != path);
    let rebuilt = last.is_some_and(|(_, revision)| revision != data.revision);
    *last = Some((path, data.revision));
    let shown = path.and_then(|p| data.path(p));
    let steps = shown.map(|p| p.steps.as_slice()).unwrap_or_default();
    let places = shown.map(|p| layout::line(&p.kinds)).unwrap_or_default();
    let mut ordered = vec![None; steps.len()];
    for (entity, mut node, mut visibility, children) in &mut nodes {
        let step = steps.iter().position(|&s| s == node.note);
        // Leaving a path: its steps float back to where they were.
        if changed && steps.is_empty() && node.target.is_some() {
            node.arriving = true;
        }
        node.hidden = !steps.is_empty() && step.is_none();
        node.target = step.and_then(|step| places.get(step).copied());
        visibility.set_if_neq(if node.hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        if let Some(step) = step {
            ordered[step] = Some(entity);
        }
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
    state.path_lines = shown
        .map(NotePath::edges)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(from, to, kind)| {
            let connection = match kind {
                StepKind::Core => Connection::Mutual,
                StepKind::Optional => Connection::OneWay,
                StepKind::Extra => Connection::Tag,
            };
            Some((ordered[from]?, ordered[to]?, connection))
        })
        .collect();
    state.path = ordered.into_iter().flatten().collect();
    if changed || (rebuilt && !state.path.is_empty()) {
        // Lay the shown notes out again, and frame them as they settle.
        state.hovered = None;
        state.alpha = 1.0;
        state.framing = true;
    }
}

/// Keeps the shown notes in frame while they settle after a path is chosen.
fn frame_path(
    mut state: ResMut<GraphState>,
    nodes: Query<(&Transform, &GraphNode), Without<GraphCamera>>,
    viewport: Query<&ComputedNode, With<GraphViewport>>,
) {
    if !state.framing {
        return;
    }
    state.camera_goal = Some(fit_all(&nodes, viewport.single().ok()));
    if state.alpha < 0.05 {
        state.framing = false;
    }
}

/// How fast (per second) path steps ease onto their line, and notes float to their homes.
const PATH_EASE: f32 = 7.0;
const ARRIVE_EASE: f32 = 2.5;
/// Pull along links, towards their length in the layout.
const LINK_PULL: f32 = 2.0;
/// Push between notes inside each other's orbits, per unit of overlap: an overlap clears
/// at about this rate (per second).
const REPEL: f32 = 6.0;
/// Velocity lost per second: high, so notes glide to rest instead of bouncing.
const DAMPING: f32 = 10.0;
/// Fastest a note floats (world units per second).
const MAX_SPEED: f32 = 2400.0;

/// Moves the shown notes. With a path followed, its steps ease quickly onto their line.
/// Otherwise notes float: no note stays inside another's orbit (a dragged note pushes others
/// smoothly out of its way, and they stay where they end up), links pull towards their
/// length in the layout (a dragged note brings its neighbours along), and notes that are
/// arriving (the graph just opened, or a path was left) float to their homes.
fn simulate(
    time: Res<Time>,
    nav: Res<Nav>,
    mut state: ResMut<GraphState>,
    mut nodes: Query<(Entity, &mut GraphNode, &mut Transform)>,
) {
    if nav.view.get_untracked() != View::Graph || state.alpha < 0.002 {
        return;
    }
    let dt = time.delta_secs().min(1.0 / 30.0);
    let following = !state.path.is_empty();
    let alpha = state.alpha;
    // Position, orbit, whether it stays put, whether it takes part in pushing.
    let mut positions: Vec<(Entity, Vec2, f32, bool, bool)> = Vec::new();
    for (entity, node, transform) in &nodes {
        if node.hidden {
            continue;
        }
        let fixed = state.dragging == Some(entity) || (node.pinned && !following);
        let pushing = !following && !node.arriving;
        positions.push((
            entity,
            transform.translation.truncate(),
            node.orbit,
            fixed,
            pushing,
        ));
    }
    let index = |e: Entity| positions.iter().position(|(p, ..)| *p == e);
    let weights = state.anchor.map(|anchor| mobility(&state.edges, anchor));
    let mobility = |e: Entity| {
        weights
            .as_ref()
            .map_or(1.0, |m| m.get(&e).copied().unwrap_or(FAR_MOBILITY))
    };
    // Links pull with the simulation's heat, nearest a dragged note most.
    let mut forces = vec![Vec2::ZERO; positions.len()];
    if !following {
        for &(a, b, rest) in &state.springs {
            let (Some(i), Some(j)) = (index(a), index(b)) else {
                continue;
            };
            let delta = positions[j].1 - positions[i].1;
            let f =
                delta.normalize_or_zero() * (delta.length() - rest) * LINK_PULL * alpha * DAMPING;
            forces[i] += f * mobility(a);
            forces[j] -= f * mobility(b);
        }
    }
    // Orbits push at any heat: whatever a dragged note comes near floats away.
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let (a, b) = (positions[i], positions[j]);
            if !(a.4 && b.4) {
                continue;
            }
            let delta = b.1 - a.1;
            let overlap = a.2 + b.2 - delta.length();
            if overlap <= 0.0 {
                continue;
            }
            let push = delta.normalize_or(Vec2::from_angle(i as f32)) * overlap * REPEL * DAMPING;
            forces[i] -= push;
            forces[j] += push;
        }
    }
    let ease = |rate: f32| 1.0 - (-rate * dt).exp();
    let mut arriving = false;
    for (i, (_, mut node, mut transform)) in
        nodes.iter_mut().filter(|(_, n, _)| !n.hidden).enumerate()
    {
        let (_, at, _, fixed, _) = positions[i];
        let next = if let (true, Some(target)) = (following, node.target) {
            node.velocity = Vec2::ZERO;
            at.lerp(target, ease(PATH_EASE))
        } else if node.arriving && !fixed {
            node.velocity = Vec2::ZERO;
            let next = at.lerp(node.home, ease(ARRIVE_EASE));
            node.arriving = next.distance(node.home) > 1.0;
            arriving |= node.arriving;
            next
        } else if fixed {
            node.velocity = Vec2::ZERO;
            at
        } else {
            // Damped: speed settles at force / DAMPING, so a force of `k · gap` closes the
            // gap at rate `k`, at any frame rate.
            node.velocity = ((node.velocity + forces[i] * dt) * (-DAMPING * dt).exp())
                .clamp_length_max(MAX_SPEED);
            at + node.velocity * dt
        };
        transform.translation.x = next.x;
        transform.translation.y = next.y;
        // Off a path, a note's home is wherever it floats to.
        if !following && !node.arriving {
            node.home = next;
        }
    }
    if state.dragging.is_none() && !arriving {
        // 1% per 60 Hz frame, at any frame rate.
        state.alpha *= 0.99_f32.powf(dt * 60.0);
    }
}

/// How freely nodes near a dragged node move, by distance in links: the node itself settles,
/// its neighbours follow partly, the next ring a little, and the rest of the graph stays put.
const NEAR_MOBILITY: [f32; 3] = [1.0, 0.3, 0.1];
const FAR_MOBILITY: f32 = 0.03;

fn mobility(
    edges: &[(Entity, Entity)],
    anchor: Entity,
) -> bevy::platform::collections::HashMap<Entity, f32> {
    let mut result = bevy::platform::collections::HashMap::default();
    result.insert(anchor, NEAR_MOBILITY[0]);
    let mut ring = vec![anchor];
    for &weight in &NEAR_MOBILITY[1..] {
        let mut next = Vec::new();
        for &(a, b) in edges {
            for (from, to) in [(a, b), (b, a)] {
                if ring.contains(&from) && !result.contains_key(&to) {
                    result.insert(to, weight);
                    next.push(to);
                }
            }
        }
        ring = next;
    }
    result
}

/// Label font size at zoom 1.
const LABEL_SIZE: f32 = 13.0;

/// Re-rasterizes labels for the current zoom so they stay sharp when zoomed in. The size
/// moves in steps of √2 (so a smooth zoom does not re-render text every frame) and the
/// label's scale compensates, keeping its size in the world unchanged.
fn sharpen_labels(
    camera: Query<&Projection, With<GraphCamera>>,
    mut texts: Query<(&GraphText, &mut TextFont, &mut Transform)>,
    mut current: Local<Option<i32>>,
) {
    let Ok(projection) = camera.single() else {
        return;
    };
    let zoom = 1.0 / ortho_scale(projection);
    let step = (zoom.log2() * 2.0).ceil().max(0.0) as i32;
    if *current == Some(step) {
        return;
    }
    *current = Some(step);
    let factor = (step as f32 / 2.0).exp2();
    for (text, mut font, mut transform) in &mut texts {
        font.font_size = (text.size * factor).into();
        transform.scale = Vec3::splat(1.0 / factor);
    }
}

fn neighbors(state: &GraphState, entity: Entity) -> Vec<Entity> {
    state
        .edges
        .iter()
        .filter_map(|&(a, b)| {
            if a == entity {
                Some(b)
            } else if b == entity {
                Some(a)
            } else {
                None
            }
        })
        .collect()
}

fn style_nodes(
    nav: Res<Nav>,
    state: Res<GraphState>,
    mut nodes: Query<(Entity, &mut GraphNode, &Children)>,
    mut icons: Query<&mut TextColor, (With<GraphIcon>, Without<GraphLabel>)>,
    mut labels: Query<&mut TextColor, (With<GraphLabel>, Without<GraphIcon>)>,
    mut fills: Query<&mut MeshMaterial2d<ColorMaterial>, (With<GraphFill>, Without<GraphRing>)>,
    mut rings: Query<
        (&mut MeshMaterial2d<ColorMaterial>, &mut Transform),
        (With<GraphRing>, Without<GraphFill>),
    >,
) {
    let current = nav.current.get_untracked();
    let focus = state.hovered.or(state.dragging);
    let focus_neighbors = focus.map(|f| neighbors(&state, f)).unwrap_or_default();
    let paint = &state.paint;
    for (entity, mut node, children) in &mut nodes {
        if node.neighbors.is_empty() {
            node.neighbors = neighbors(&state, entity);
        }
        let lit = focus.is_none_or(|f| f == entity || focus_neighbors.contains(&entity));
        let is_current = node.note == current;
        let icon = if is_current {
            color::BACKGROUND
        } else if Some(entity) == focus {
            HOVERED_COLOR
        } else if !lit {
            DIM_COLOR
        } else {
            NOTE_COLOR
        };
        let label = if is_current || Some(entity) == focus {
            color::FOREGROUND
        } else if lit {
            color::MUTED_FOREGROUND
        } else {
            color::SUBTLE_FOREGROUND
        };
        let (ring, ring_radius) = if is_current {
            (&paint.current_ring, node.radius + HALO)
        } else if Some(entity) == focus {
            (&paint.ring_hovered, node.radius)
        } else {
            (&paint.ring, node.radius)
        };
        let fill = if is_current {
            &paint.current_fill
        } else {
            &paint.fill
        };
        for &child in children {
            if let Ok(mut color) = icons.get_mut(child) {
                color.set_if_neq(TextColor(icon));
            } else if let Ok(mut color) = labels.get_mut(child) {
                color.set_if_neq(TextColor(label));
            } else if let Ok(mut material) = fills.get_mut(child) {
                if material.0 != *fill {
                    material.0 = fill.clone();
                }
            } else if let Ok((mut material, mut transform)) = rings.get_mut(child) {
                if material.0 != *ring {
                    material.0 = ring.clone();
                }
                let scale = Vec3::splat(ring_radius);
                if transform.scale != scale {
                    transform.scale = scale;
                }
            }
        }
    }
}

/// How two notes are connected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    /// Each links to the other: a solid line.
    Mutual,
    /// One links to the other: a dashed line.
    OneWay,
    /// They share a tag but neither links to the other: a dotted line.
    Tag,
}

/// Tags on more notes than this draw no lines: they would connect everything to everything.
const MAX_TAG_NOTES: usize = 12;

/// The connected pairs of notes (each pair once, lower id first) and how they connect.
pub fn connections(vault: &VaultData) -> Vec<(NoteId, NoteId, Connection)> {
    let mut lines = std::collections::BTreeMap::new();
    for (a, note) in vault.notes.iter().enumerate() {
        for &b in &note.outgoing {
            let mutual = vault.notes[b].outgoing.contains(&a);
            let kind = if mutual {
                Connection::Mutual
            } else {
                Connection::OneWay
            };
            lines.insert((a.min(b), a.max(b)), kind);
        }
    }
    for notes in vault.tags.values().filter(|n| n.len() <= MAX_TAG_NOTES) {
        for (i, &a) in notes.iter().enumerate() {
            for &b in &notes[i + 1..] {
                if a != b {
                    lines.entry((a.min(b), a.max(b))).or_insert(Connection::Tag);
                }
            }
        }
    }
    lines
        .into_iter()
        .map(|((a, b), kind)| (a, b, kind))
        .collect()
}

/// Line width on screen, in render-target pixels.
const EDGE_WIDTH: f32 = 4.0;
/// Width of the lines of the whole graph (not hovered), as a share of [`EDGE_WIDTH`].
const THIN: f32 = 0.55;
/// Dashes of one-way links and the gaps between them, in render-target pixels.
const DASH: f32 = 12.0;
const DASH_GAP: f32 = 8.0;
/// Space between the dots of tag lines, in render-target pixels.
const DOT_GAP: f32 = 6.0;

/// Pieces `piece` long, `gap` apart, centred on `0..length` (start and end of each).
fn pattern(length: f32, piece: f32, gap: f32) -> impl Iterator<Item = (f32, f32)> {
    let count = ((length + gap) / (piece + gap)).floor().max(0.0) as usize;
    let used = count as f32 * (piece + gap) - gap;
    let start = (length - used) / 2.0;
    (0..count).map(move |i| {
        let from = start + i as f32 * (piece + gap);
        (from, from + piece)
    })
}

/// Vertices, colours and indices of the line mesh.
#[derive(Default)]
struct LineMesh {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl LineMesh {
    /// A line between two discs (centre, radius) in the style of `kind`; `scale` is the
    /// half width and the world units per pixel.
    fn line(
        &mut self,
        kind: Connection,
        (pa, ra): (Vec2, f32),
        (pb, rb): (Vec2, f32),
        (half, px): (f32, f32),
        color: [f32; 4],
    ) {
        // Patterns run between the discs' outlines, so both ends look the same.
        let direction = (pb - pa).normalize_or_zero();
        let from = pa + direction * ra;
        let length = (pb - direction * rb - from).dot(direction);
        match kind {
            Connection::Mutual => self.quad(pa, pb, half, color),
            Connection::OneWay => {
                for (start, end) in pattern(length, DASH * px, DASH_GAP * px) {
                    self.quad(
                        from + direction * start,
                        from + direction * end,
                        half,
                        color,
                    );
                }
            }
            Connection::Tag => {
                for (start, end) in pattern(length, EDGE_WIDTH * px, DOT_GAP * px) {
                    self.dot(from + direction * (start + end) / 2.0, half, color);
                }
            }
        }
    }

    fn quad(&mut self, from: Vec2, to: Vec2, half: f32, color: [f32; 4]) {
        let normal = (to - from).normalize_or_zero().perp() * half;
        let base = self.positions.len() as u32;
        for p in [from + normal, from - normal, to - normal, to + normal] {
            self.positions.push([p.x, p.y, 0.0]);
            self.colors.push(color);
        }
        self.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// A round dot (an octagon).
    fn dot(&mut self, center: Vec2, radius: f32, color: [f32; 4]) {
        let base = self.positions.len() as u32;
        self.positions.push([center.x, center.y, 0.0]);
        self.colors.push(color);
        for i in 0..8 {
            let p = center + Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4) * radius;
            self.positions.push([p.x, p.y, 0.0]);
            self.colors.push(color);
            self.indices
                .extend([base, base + 1 + i, base + 1 + (i + 1) % 8]);
        }
    }
}

fn draw_edges(
    nav: Res<Nav>,
    state: Res<GraphState>,
    nodes: Query<(&Transform, &GraphNode)>,
    camera: Query<&Projection, With<GraphCamera>>,
    edge_mesh: Query<&Mesh2d, With<EdgeMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if nav.view.get_untracked() != View::Graph {
        return;
    }
    let (Ok(mesh), Ok(projection)) = (edge_mesh.single(), camera.single()) else {
        return;
    };
    // World units per render-target pixel: lines and patterns keep their size on screen.
    let px = ortho_scale(projection);
    let half = EDGE_WIDTH / 2.0 * px;
    let focus = state.hovered.or(state.dragging);
    let mut out = LineMesh::default();
    // Following a path: only its steps, joined as the path says.
    let path_color = color::LINK.to_linear().to_f32_array();
    for &(a, b, kind) in &state.path_lines {
        if let (Ok((ta, na)), Ok((tb, nb))) = (nodes.get(a), nodes.get(b)) {
            out.line(
                kind,
                (ta.translation.truncate(), na.radius),
                (tb.translation.truncate(), nb.radius),
                (half, px),
                path_color,
            );
        }
    }
    let lines = if state.path.is_empty() {
        state.lines.as_slice()
    } else {
        &[]
    };
    for &(a, b, kind, overview) in lines {
        let highlighted = focus.is_some_and(|f| f == a || f == b);
        if !overview && !highlighted {
            continue;
        }
        let (Ok((ta, na)), Ok((tb, nb))) = (nodes.get(a), nodes.get(b)) else {
            continue;
        };
        // Many lines cross the whole graph: thin and faint, except a hovered note's.
        let (color, width) = if highlighted {
            (color::LINK, half)
        } else if focus.is_some() {
            (Color::srgb(0.09, 0.09, 0.09), half * THIN)
        } else {
            (Color::srgb(0.17, 0.17, 0.17), half * THIN)
        };
        out.line(
            kind,
            (ta.translation.truncate(), na.radius),
            (tb.translation.truncate(), nb.radius),
            (width, px),
            color.to_linear().to_f32_array(),
        );
    }
    if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, out.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, out.colors);
        mesh.insert_indices(bevy::mesh::Indices::U32(out.indices));
    }
}

// ---------------------------------------------------------------------------
// Pointer input on the viewport image
// ---------------------------------------------------------------------------

/// Maps a window-space pointer position to graph world space.
fn to_world(
    pointer: Vec2,
    viewport: Entity,
    nodes_ui: &Query<(&ComputedNode, &UiGlobalTransform)>,
    camera: &Query<(&Camera, &GlobalTransform, &Projection), With<GraphCamera>>,
) -> Option<Vec2> {
    let (node, transform) = nodes_ui.get(viewport).ok()?;
    let (_, cam_transform, projection) = camera.single().ok()?;
    let scale = match projection {
        Projection::Orthographic(o) => o.scale,
        _ => 1.0,
    };
    // Pointer in UI physical space, then relative to the viewport centre.
    let local = transform
        .try_inverse()?
        .transform_point2(pointer / node.inverse_scale_factor);
    // The render target is sized in physical px of the node, and the camera maps one target
    // pixel to one world unit at scale 1.
    let world = Vec2::new(local.x, -local.y) * scale;
    Some(cam_transform.translation().truncate() + world)
}

fn pick(world: Vec2, nodes: &Query<(Entity, &Transform, &GraphNode)>) -> Option<Entity> {
    nodes
        .iter()
        .filter(|(_, t, n)| {
            !n.hidden && t.translation.truncate().distance(world) <= n.radius + 12.0
        })
        .min_by(|a, b| {
            let da = a.1.translation.truncate().distance(world);
            let db = b.1.translation.truncate().distance(world);
            da.total_cmp(&db)
        })
        .map(|(e, ..)| e)
}

fn on_move(
    event: On<PointerMove>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<GraphCamera>>,
    nodes: Query<(Entity, &Transform, &GraphNode)>,
    mut state: ResMut<GraphState>,
    mut cursor: Query<&mut bevy::picking::cursor::EntityCursor>,
) {
    if state.dragging.is_some() {
        return;
    }
    let Some(world) = to_world(event.pointer.position, event.entity, &nodes_ui, &camera) else {
        return;
    };
    let hovered = pick(world, &nodes);
    if state.hovered != hovered {
        state.hovered = hovered;
    }
    let icon = if hovered.is_some() {
        bevy::window::SystemCursorIcon::Pointer
    } else {
        bevy::window::SystemCursorIcon::Grab
    };
    let wanted = bevy::picking::cursor::EntityCursor::System(icon);
    if let Ok(mut c) = cursor.get_mut(event.entity) {
        if *c != wanted {
            *c = wanted;
        }
    } else {
        // First move: the viewport has no cursor component yet.
    }
}

fn on_out(_: On<PointerOut>, mut state: ResMut<GraphState>) {
    if state.dragging.is_none() {
        state.hovered = None;
    }
}

fn on_press(
    event: On<PointerPress>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<GraphCamera>>,
    nodes: Query<(Entity, &Transform, &GraphNode)>,
    mut state: ResMut<GraphState>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    // Touch has no hover, so pick here as well.
    let node = to_world(event.pointer.position, event.entity, &nodes_ui, &camera)
        .and_then(|world| pick(world, &nodes));
    state.hovered = node;
    state.press = Some(Press { node, moved: false });
    state.pan_velocity = Vec2::ZERO;
    state.camera_goal = None;
    state.framing = false;
}

fn on_release(event: On<PointerRelease>, mut state: ResMut<GraphState>) {
    // A finger leaves no hover behind.
    if event.pointer.id.is_touch() && state.dragging.is_none() {
        state.hovered = None;
    }
}

fn on_click(
    event: On<PointerClick>,
    state: Res<GraphState>,
    nodes: Query<&GraphNode>,
    nav: Res<Nav>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    // Only a press that stayed put is a click; dragging a node or panning never opens it.
    let Some(Press {
        node: Some(entity),
        moved: false,
    }) = state.press
    else {
        return;
    };
    if let Ok(node) = nodes.get(entity) {
        nav.open(node.note, None);
    }
}

fn on_drag(
    event: On<PointerDrag>,
    time: Res<Time<Real>>,
    touches: Res<Touches>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut camera: Query<(&Camera, &mut Transform, &Projection), With<GraphCamera>>,
    mut nodes: Query<&mut Transform, (With<GraphNode>, Without<GraphCamera>)>,
    mut state: ResMut<GraphState>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Some(mut press) = state.press else {
        return;
    };
    // Two fingers pinch instead (see `pinch_zoom`).
    if touches.iter().count() > 1 {
        return;
    }
    if !press.moved {
        let slop = if event.pointer.id.is_touch() {
            TOUCH_CLICK_SLOP
        } else {
            CLICK_SLOP
        };
        if event.distance.length() < slop {
            return;
        }
        press.moved = true;
        state.press = Some(press);
        state.dragging = press.node;
        if press.node.is_some() {
            state.anchor = press.node;
            state.alpha = state.alpha.max(0.15);
        }
    }
    let Ok((node, _)) = nodes_ui.get(event.entity) else {
        return;
    };
    let Ok((_, mut cam, projection)) = camera.single_mut() else {
        return;
    };
    // Logical px → target px → world units.
    let delta = event.delta / node.inverse_scale_factor * ortho_scale(projection);
    let delta = Vec2::new(delta.x, -delta.y);
    if let Some(entity) = state.dragging {
        if let Ok(mut transform) = nodes.get_mut(entity) {
            transform.translation += delta.extend(0.0);
        }
    } else {
        cam.translation -= delta.extend(0.0);
        // Real frame time: a lower cap would overestimate the fling speed on slow frames.
        let dt = time.delta_secs().clamp(1.0 / 240.0, 0.1);
        state.pan_velocity = state.pan_velocity.lerp(-delta / dt, 0.4);
    }
}

fn on_drag_end(_: On<PointerDragEnd>, mut state: ResMut<GraphState>) {
    // A fling keeps panning (see `camera_motion`); a dragged node just stops, and stays
    // where it was dropped (its home follows it, see `simulate`).
    let dropped = state.dragging.take().is_some();
    if dropped || !state.press.is_some_and(|p| p.moved) {
        state.pan_velocity = Vec2::ZERO;
    }
}

fn on_scroll(
    event: On<PointerScroll>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut camera: Query<(&Camera, &mut Transform, &mut Projection), With<GraphCamera>>,
    mut state: ResMut<GraphState>,
) {
    let Some(anchor) = viewport_to_world(event.pointer.position, event.entity, &nodes_ui, &camera)
    else {
        return;
    };
    let Ok((_, mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    state.camera_goal = None;
    state.framing = false;
    let factor = (1.0 - event.y.signum() * 0.12).clamp(0.5, 1.5);
    zoom_around(&mut transform, &mut projection, anchor, factor);
}

/// Like [`to_world`], for a camera query with mutable access.
fn viewport_to_world(
    pointer: Vec2,
    viewport: Entity,
    nodes_ui: &Query<(&ComputedNode, &UiGlobalTransform)>,
    camera: &Query<(&Camera, &mut Transform, &mut Projection), With<GraphCamera>>,
) -> Option<Vec2> {
    let (node, transform) = nodes_ui.get(viewport).ok()?;
    let (_, cam, projection) = camera.single().ok()?;
    let local = transform
        .try_inverse()?
        .transform_point2(pointer / node.inverse_scale_factor);
    Some(cam.translation.truncate() + Vec2::new(local.x, -local.y) * ortho_scale(projection))
}

fn ortho_scale(projection: &Projection) -> f32 {
    match projection {
        Projection::Orthographic(o) => o.scale,
        _ => 1.0,
    }
}

const MIN_SCALE: f32 = 0.25;
const MAX_SCALE: f32 = 6.0;
/// Closest zoom "fit" goes to.
const FIT_MIN_SCALE: f32 = 0.8;

/// Zooms by `factor` (< 1 zooms in) keeping the world point `anchor` fixed on screen.
fn zoom_around(transform: &mut Transform, projection: &mut Projection, anchor: Vec2, factor: f32) {
    let Projection::Orthographic(o) = projection else {
        return;
    };
    let scale = (o.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
    let applied = scale / o.scale;
    o.scale = scale;
    let camera = transform.translation.truncate();
    transform.translation = (anchor - (anchor - camera) * applied).extend(transform.translation.z);
}

/// Two-finger pinch zooms around the point between the fingers.
fn pinch_zoom(
    nav: Res<Nav>,
    touches: Res<Touches>,
    viewport: Query<Entity, With<GraphViewport>>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut camera: Query<(&Camera, &mut Transform, &mut Projection), With<GraphCamera>>,
    mut state: ResMut<GraphState>,
) {
    let fingers: Vec<Vec2> = touches.iter().map(Touch::position).collect();
    let (true, [a, b], Ok(viewport)) = (
        nav.view.get_untracked() == View::Graph,
        fingers.as_slice(),
        viewport.single(),
    ) else {
        state.pinch = None;
        return;
    };
    let distance = a.distance(*b).max(1.0);
    if let Some(previous) = state.pinch
        && let Some(anchor) = viewport_to_world((a + b) / 2.0, viewport, &nodes_ui, &camera)
        && let Ok((_, mut transform, mut projection)) = camera.single_mut()
    {
        zoom_around(&mut transform, &mut projection, anchor, previous / distance);
    }
    state.pinch = Some(distance);
    state.dragging = None;
    state.pan_velocity = Vec2::ZERO;
    state.camera_goal = None;
    state.framing = false;
    // A pinch is never a click.
    if let Some(press) = &mut state.press {
        press.moved = true;
    }
}

/// Panning momentum and eased camera moves.
fn camera_motion(
    time: Res<Time<Real>>,
    mut state: ResMut<GraphState>,
    mut camera: Query<(&mut Transform, &mut Projection), With<GraphCamera>>,
) {
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(1.0 / 30.0);
    if state.pan_velocity != Vec2::ZERO && state.dragging.is_none() {
        transform.translation += (state.pan_velocity * dt).extend(0.0);
        state.pan_velocity *= (-5.0 * dt).exp();
        if state.pan_velocity.length() < 5.0 {
            state.pan_velocity = Vec2::ZERO;
        }
    }
    if let Some((goal, goal_scale)) = state.camera_goal
        && let Projection::Orthographic(o) = &mut *projection
    {
        let t = 1.0 - (-14.0 * dt).exp();
        let position = transform.translation.truncate().lerp(goal, t);
        transform.translation = position.extend(transform.translation.z);
        o.scale += (goal_scale - o.scale) * t;
        if position.distance(goal) < 0.5 && (o.scale - goal_scale).abs() < 0.002 {
            transform.translation = goal.extend(transform.translation.z);
            o.scale = goal_scale;
            state.camera_goal = None;
        }
    }
}

/// Camera position and zoom that show every node.
fn fit_all(
    nodes: &Query<(&Transform, &GraphNode), Without<GraphCamera>>,
    viewport: Option<&ComputedNode>,
) -> (Vec2, f32) {
    let mut bounds: Option<Rect> = None;
    for (transform, _) in nodes.iter().filter(|(_, n)| !n.hidden) {
        let p = transform.translation.truncate();
        // Leave room for the node and its label below.
        let r = Rect::from_center_half_size(p, Vec2::new(80.0, 50.0));
        bounds = Some(bounds.map_or(r, |b| b.union(r)));
    }
    let (Some(bounds), Some(viewport)) = (bounds, viewport) else {
        return (Vec2::ZERO, 1.0);
    };
    let size = viewport.size().max(Vec2::ONE);
    let scale = (bounds.size() / size).max_element() * 1.2;
    // A few notes are framed at a comfortable size, not blown up to fill the view.
    (bounds.center(), scale.clamp(FIT_MIN_SCALE, MAX_SCALE))
}

/// Zoom buttons over the graph (the only way to zoom on touch screens): one bordered group of
/// zoom in, zoom out and fit.
pub fn zoom_controls() -> impl Scene {
    fn button(icon: &'static str, tip: &'static str, factor: f32, first: bool) -> impl Scene {
        bsn! {
            @{icon_button(Variant::Ghost, Size::Icon, icon)}
            Tooltip::new(tip)
            Node {
                border: UiRect::top(px(if first { 0.0 } else { 1.0 })),
                border_radius: BorderRadius::ZERO,
            }
            BorderColor::all(color::BORDER)
            on(move |_: On<bevy::ui_widgets::Activate>,
                     camera: Query<(&Transform, &Projection), With<GraphCamera>>,
                     nodes: Query<(&Transform, &GraphNode), Without<GraphCamera>>,
                     viewport: Query<&ComputedNode, With<GraphViewport>>,
                     mut state: ResMut<GraphState>| {
                let Ok((transform, projection)) = camera.single() else {
                    return;
                };
                let (position, scale) = state
                    .camera_goal
                    .unwrap_or((transform.translation.truncate(), ortho_scale(projection)));
                state.pan_velocity = Vec2::ZERO;
                state.framing = false;
                state.camera_goal = Some(if factor == 0.0 {
                    fit_all(&nodes, viewport.single().ok())
                } else {
                    (position, (scale * factor).clamp(MIN_SCALE, MAX_SCALE))
                });
            })
        }
    }
    let shadow = shadow::popover();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            right: px(16),
            bottom: px(16),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::LG)),
            overflow: Overflow::clip(),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::INPUT)
        shadow
        Children [
            @{button("plus", "Zoom in", 0.8, true)}
            --
            @{button("minus", "Zoom out", 1.25, false)}
            --
            @{button("maximize", "Fit to screen", 0.0, false)}
        ]
    }
}

/// A row of the legend: of the line styles of links, or of the steps of a followed path.
#[derive(Component, Clone, Copy, Debug, Default)]
struct LegendRow {
    path: bool,
}

fn show_legend(state: Res<GraphState>, mut rows: Query<(&LegendRow, &mut Node)>) {
    let following = !state.path.is_empty();
    for (row, mut node) in &mut rows {
        let display = if row.path == following {
            Display::Flex
        } else {
            Display::None
        };
        node.display = display;
    }
}

/// What the line styles mean, in the graph's corner.
pub fn legend() -> impl Scene {
    fn row(sample: Vec<Box<dyn Scene>>, text: &str, path: bool) -> impl Scene + use<> {
        bsn! {
            Node { align_items: AlignItems::Center, column_gap: px(10) }
            LegendRow { path: path }
            Children [
                Node { width: px(24), height: px(6), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween }
                Children [ {sample} ]
                --
                @{caption(text)}
            ]
        }
    }
    fn piece(width: f32, round: bool) -> Box<dyn Scene> {
        let node = Node {
            width: px(width),
            height: px(if round { 4.0 } else { 2.0 }),
            border_radius: BorderRadius::all(px(if round { 2.0 } else { 0.0 })),
            ..default()
        };
        Box::new(bsn! { node BackgroundColor(color::MUTED_FOREGROUND) })
    }
    let shadow = shadow::popover();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            bottom: px(16),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            padding: UiRect::axes(px(12), px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::LG)),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::INPUT)
        Pickable::IGNORE
        shadow
        Children [
            @{row(vec![piece(24.0, false)], "Linked both ways", false)}
            --
            @{row((0..3).map(|_| piece(6.0, false)).collect(), "Linked one way", false)}
            --
            @{row((0..4).map(|_| piece(4.0, true)).collect(), "Shared tag", false)}
            --
            @{row(vec![piece(24.0, false)], "Core step", true)}
            --
            @{row((0..3).map(|_| piece(6.0, false)).collect(), "Optional step", true)}
            --
            @{row((0..4).map(|_| piece(4.0, true)).collect(), "Extra step", true)}
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dragged_note_pushes_others_away_and_they_stay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(Nav::new(0))
            .insert_resource(GraphState {
                alpha: 0.15,
                ..default()
            })
            .add_systems(Update, simulate);
        app.world().resource::<Nav>().view.set(View::Graph);
        let node = |at: Vec2| {
            (
                GraphNode {
                    note: 0,
                    velocity: Vec2::ZERO,
                    radius: 20.0,
                    pinned: false,
                    neighbors: Vec::new(),
                    hidden: false,
                    home: at,
                    arriving: false,
                    target: None,
                    orbit: 50.0,
                },
                Transform::from_translation(at.extend(0.0)),
            )
        };
        let dragged = app.world_mut().spawn(node(Vec2::ZERO)).id();
        let other = app.world_mut().spawn(node(Vec2::new(30.0, 0.0))).id();
        app.world_mut().resource_mut::<GraphState>().dragging = Some(dragged);
        for _ in 0..120 {
            app.update();
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        let at = |app: &App, e: Entity| app.world().get::<Transform>(e).unwrap().translation;
        assert_eq!(
            at(&app, dragged).truncate(),
            Vec2::ZERO,
            "the dragged note stays put"
        );
        let pushed = at(&app, other).x;
        assert!(
            pushed > 80.0,
            "pushed out of the dragged note's orbit: {pushed}"
        );
        let home = app.world().get::<GraphNode>(other).unwrap().home;
        assert!((home.x - pushed).abs() < 1.0, "its new place is its home");
    }

    #[test]
    fn connections_follow_links_and_tags() {
        let vault = VaultData::from_sources([
            ("A.md", "# A\n\n[[B]] [[C]] #shared"),
            ("B.md", "# B\n\n[[A]]"),
            ("C.md", "# C"),
            ("D.md", "# D\n\n#shared"),
        ]);
        assert_eq!(
            connections(&vault),
            vec![
                (0, 1, Connection::Mutual),
                (0, 2, Connection::OneWay),
                (0, 3, Connection::Tag),
            ]
        );
    }

    #[test]
    fn patterns_are_centred() {
        let pieces: Vec<_> = pattern(50.0, 10.0, 5.0).collect();
        assert_eq!(pieces, vec![(5.0, 15.0), (20.0, 30.0), (35.0, 45.0)]);
        assert_eq!(pattern(5.0, 10.0, 5.0).count(), 0);
    }
}
