//! The graph view, drawn as a hand-made game map.
//!
//! Every note sits where the vault's map note puts it ([`bevsidian_core::map`]): Bevy, the
//! capital, in the middle, the main quest on a ring road around it and a region for every
//! area. Notes are drawn as their icons inside a shape that says what they are (a hexagon
//! for an area's town, a diamond for a milestone, a star for a route's goal, a square for a
//! crate…). Reading paths are roads ([`roads`], [`routes`]): the map shows the main quest,
//! the routes the reader chose in its checklist and the route being followed; everything else
//! waits, folded, in the note it hangs from ([`fold`]) or shows with the whole map.
//!
//! The map lives in a 2D world on its own render layer. A dedicated camera renders that world
//! into an image that the UI shows in the [`GraphViewport`] node; pointer events on the node
//! are mapped back into the world for hovering, clicking, panning (with momentum) and zooming
//! (wheel, pinch and buttons). A press only opens a note if the pointer did not move: drags
//! never click. While the map is edited ([`edit`]), dragging moves notes and objects instead
//! and the changes are saved into the map note.

use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    input::touch::{Touch, Touches},
    picking::events::{
        PointerClick, PointerDrag, PointerDragEnd, PointerMove, PointerOut, PointerPress,
        PointerRelease, PointerScroll,
    },
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
    text::{FontWeight, LineHeight},
    ui::UiGlobalTransform,
};

use bevsidian_core::{
    map::{Scale, Shape},
    nav::{Nav, View},
    vault::{NoteId, Vault, VaultData},
};
use bevsidian_ui::color;

mod controls;
mod edit;
mod fold;
mod hint;
mod roads;
mod routes;
mod shapes;
mod tree;
mod world;

pub use controls::{edit_bar, legend, zoom_controls};
pub use edit::MapEdited;
pub use hint::hover_hint;

const LAYER: usize = 1;

/// Adds the graph view; place a [`GraphViewport`] node in the UI to show it.
pub fn plugin(app: &mut App) {
    app.init_resource::<GraphState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_nodes,
                world::spawn_world,
                routes::choose_routes,
                fold::fold_nodes,
                edit::edit_map,
                controls::show_legend,
                hint::show_hint,
                attach_viewport,
                toggle_camera,
                resize_target,
            )
                .chain()
                .in_set(GraphSystems::Build),
        )
        .add_systems(
            Update,
            (
                motion,
                frame,
                pinch_zoom,
                camera_motion,
                sharpen_labels,
                style_nodes,
                world::show_handles,
                world::scale_regions,
                world::draw_ground,
                roads::draw_roads,
            )
                .chain()
                .after(GraphSystems::Build),
        );
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
enum GraphSystems {
    Build,
}

/// UI node that displays the graph.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct GraphViewport;

#[derive(Component)]
struct GraphCamera;

/// A note on the map, drawn as its icon in its shape, with its name below.
#[derive(Component)]
#[allow(clippy::struct_excessive_bools)] // Independent states of one note, not a mode.
struct GraphNode {
    note: NoteId,
    /// Its size: the radius of its shape.
    radius: f32,
    /// How deep it sits in the tree (see [`tree::rank`]); 0 for the hub.
    rank: usize,
    /// On a route the map shows: always on the map, whatever is folded.
    forced: bool,
    /// Its place on the followed route (from 0), if it is on it.
    step: Option<usize>,
    /// The note it hangs from (`None` for the hub).
    parent: Option<Entity>,
    /// How many notes it holds folded (see [`fold`]); 0 for none.
    folds: usize,
    /// The notes it holds are unfolded.
    open: bool,
    /// Not drawn or picked: folded away, or off the routes shown.
    hidden: bool,
    /// Floating into its parent (to `tuck_to`) before it is folded away.
    tucking: bool,
    tuck_to: Vec2,
    /// Its place on the map.
    home: Vec2,
    /// Floating to its home (the map just opened, or it unfolded).
    arriving: bool,
}

#[derive(Component)]
struct GraphLabel;

#[derive(Component)]
struct GraphIcon;

/// The fill of a node's shape.
#[derive(Component)]
struct GraphFill;

/// The outline of a node's shape.
#[derive(Component)]
struct GraphRing;

/// The tick on a note the reader has opened.
#[derive(Component)]
struct GraphVisited;

/// Text drawn in the graph world, re-rasterized as the camera zooms so it stays sharp.
#[derive(Component)]
struct GraphText {
    /// Size in world units.
    size: f32,
}

#[derive(Resource, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct GraphState {
    image: Handle<Image>,
    /// Links, one per direction (a hovered note shows its own).
    edges: Vec<(Entity, Entity)>,
    /// The roads of every route.
    roads: Vec<roads::Road>,
    /// Each note and the note it hangs from: a faint trail where no road reaches it.
    trails: Vec<(Entity, Entity)>,
    /// The routes shown (their path notes), the one followed, and whether the whole map is.
    active: Vec<NoteId>,
    followed: Option<NoteId>,
    whole: bool,
    /// The map is being edited (every note and road shows).
    editing: bool,
    /// What shows changed: fold the map again.
    refold: bool,
    /// The camera frames the shown notes (all of them, or the followed route's) until the
    /// user moves it.
    framing: bool,
    frame_route: bool,
    hovered: Option<Entity>,
    /// The note or map object being dragged (objects and notes move only while editing).
    dragging: Option<Entity>,
    /// The current press, until the next one.
    press: Option<Press>,
    /// Panning momentum after a fling, in world units per second.
    pan_velocity: Vec2,
    /// Camera position and zoom being eased towards (zoom buttons, "Fit").
    camera_goal: Option<(Vec2, f32)>,
    /// Distance between two fingers in the previous frame of a pinch.
    pinch: Option<f32>,
    /// Some note is still floating.
    moving: bool,
    /// A note to fold or unfold, and a request to unfold (`true`) or fold every note.
    fold_toggle: Option<Entity>,
    fold_all: Option<bool>,
    /// The node clicked last, to tell a double click on it.
    last_click: Option<Entity>,
    /// While editing: the note or object chosen, and the one just dropped (to save).
    selected: Option<Entity>,
    dropped: Option<Entity>,
    /// A change asked for in the edit bar.
    edit_request: Option<edit::EditRequest>,
    /// The land moved: redraw the ground.
    ground_dirty: bool,
    /// Shared meshes and materials.
    circle: Handle<Mesh>,
    shapes: HashMap<Shape, Handle<Mesh>>,
    paint: NodePaint,
}

/// Materials of a node's fill and outline, per state.
#[derive(Default)]
struct NodePaint {
    fill: Handle<ColorMaterial>,
    ring: Handle<ColorMaterial>,
    ring_hovered: Handle<ColorMaterial>,
    current_fill: Handle<ColorMaterial>,
    current_ring: Handle<ColorMaterial>,
    selected_ring: Handle<ColorMaterial>,
    /// The badge of a note holding folded notes, and of one whose notes are unfolded.
    badge_folded: Handle<ColorMaterial>,
    badge_open: Handle<ColorMaterial>,
    /// The tick on a visited note.
    visited: Handle<ColorMaterial>,
}

impl GraphState {
    /// Whether anything on the map still needs frames.
    pub fn animating(&self) -> bool {
        self.moving
            || self.framing
            || self.dragging.is_some()
            || self.pan_velocity != Vec2::ZERO
            || self.camera_goal.is_some()
            || self.pinch.is_some()
    }
}

/// A primary-button press on the graph.
#[derive(Clone, Copy, Debug)]
struct Press {
    /// Node (or, while editing, map object) under the pointer when pressed.
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
    for (marker, z) in [(0, 0.0), (1, 0.0)] {
        let mut entity = commands.spawn((
            Mesh2d(meshes.add(world::empty_mesh())),
            // Opaque, so depth decides what covers what: the ground under the roads, the
            // roads under the notes and their names.
            MeshMaterial2d(materials.add(ColorMaterial {
                alpha_mode: bevy::sprite_render::AlphaMode2d::Opaque,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, z),
            RenderLayers::layer(LAYER),
        ));
        if marker == 0 {
            entity.insert(roads::RoadMesh);
        } else {
            entity.insert(world::Ground);
        }
    }

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

    state.paint = NodePaint {
        fill: materials.add(color::CARD),
        ring: materials.add(RING_COLOR),
        ring_hovered: materials.add(color::MUTED_FOREGROUND),
        current_fill: materials.add(color::LINK),
        current_ring: materials.add(color::LINK_RING),
        selected_ring: materials.add(color::LINK),
        badge_folded: materials.add(color::FOREGROUND),
        badge_open: materials.add(color::CARD),
        visited: materials.add(color::SUCCESS),
    };
    state.circle = meshes.add(Circle::new(1.0));
    state.shapes = Shape::ALL
        .into_iter()
        .map(|shape| (shape, meshes.add(shapes::mesh(shape))))
        .collect();
}

/// The shape a note is drawn in: the map's choice, or what its kind gives it.
fn default_shape(rank: usize) -> Shape {
    match rank {
        0 => Shape::Octagon,
        1 => Shape::Hexagon,
        4 => Shape::Square,
        _ => Shape::Circle,
    }
}

/// A note's radius: by its shape and rank, then the map's size.
fn node_radius(shape: Shape, rank: usize, scale: Scale) -> f32 {
    let base = match (shape, rank) {
        (Shape::Octagon, _) => 54.0,
        (Shape::Hexagon, _) => 37.0,
        (Shape::Star, _) => 34.0,
        (Shape::Diamond, _) => 31.0,
        (Shape::Pentagon, _) => 30.0,
        (Shape::Square, _) | (_, 4) => 21.0,
        (_, 3) => 23.0,
        _ => 26.0,
    };
    base * scale.factor()
}

/// How far a shape reaches below its centre, as a share of its radius (where the label goes).
fn reach(shape: Shape) -> f32 {
    shapes::outline(shape)
        .iter()
        .map(|p| -p.y)
        .fold(0.0, f32::max)
}

/// (Re)builds the nodes, roads and trails from the vault, at startup and after a note (or
/// the map) is edited.
fn spawn_nodes(
    mut commands: Commands,
    vault: Res<Vault>,
    mut state: ResMut<GraphState>,
    old: Query<(Entity, &GraphNode)>,
    mut built: Local<Option<u64>>,
) {
    let vault = vault.get_untracked();
    if *built == Some(vault.revision) {
        return;
    }
    let first = built.is_none();
    *built = Some(vault.revision);
    let mut was_open = std::collections::HashSet::new();
    for (entity, node) in &old {
        if node.open {
            was_open.insert(node.note);
        }
        commands.entity(entity).despawn();
    }
    state.hovered = None;
    state.dragging = None;
    state.press = None;
    state.selected = None;
    let layout = home_positions(&vault);
    let font = |family: &'static str, size: f32, weight: FontWeight| TextFont {
        font: bevy::text::FontSource::families(family),
        font_size: size.into(),
        weight,
        ..default()
    };
    // Entities first, so each node can name the node it hangs from.
    let entities: Vec<Entity> = layout
        .notes
        .iter()
        .map(|_| commands.spawn_empty().id())
        .collect();
    let entity_of = |id: NoteId| {
        layout
            .notes
            .iter()
            .position(|&n| n == id)
            .map(|s| entities[s])
    };
    for (slot, &id) in layout.notes.iter().enumerate() {
        let note = &vault.notes[id];
        let rank = layout.rank[slot];
        let place = vault.map.as_ref().and_then(|m| m.place(&note.name));
        let shape = place
            .and_then(|p| p.shape)
            .unwrap_or_else(|| default_shape(rank));
        let radius = node_radius(shape, rank, place.map_or(Scale::Normal, |p| p.scale));
        let home = layout.homes[slot];
        let icon_size = (radius * 0.9).round();
        let mesh = state.shapes.get(&shape).cloned().unwrap_or_default();
        let (fill, ring) = (state.paint.fill.clone(), state.paint.ring.clone());
        let label_size = if rank <= 1 {
            LABEL_SIZE + 1.0
        } else {
            LABEL_SIZE
        };
        let weight = if rank <= 1 {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        };
        commands
            .entity(entities[slot])
            .insert((
                GraphNode {
                    note: id,
                    radius,
                    rank,
                    forced: rank == 0,
                    step: None,
                    parent: layout.parent[slot].map(|p| entities[p]),
                    folds: 0,
                    open: was_open.contains(&id),
                    hidden: false,
                    tucking: false,
                    tuck_to: Vec2::ZERO,
                    home,
                    arriving: first,
                },
                // The first time, notes float out from the capital to their places.
                Transform::from_translation(
                    if first { home * START_SPREAD } else { home }.extend(1.0),
                ),
                Visibility::default(),
                RenderLayers::layer(LAYER),
            ))
            .with_children(|node| {
                node.spawn((
                    GraphRing,
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(ring),
                    Transform::from_xyz(0.0, 0.0, -0.2).with_scale(Vec3::splat(radius)),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphFill,
                    Mesh2d(mesh),
                    MeshMaterial2d(fill),
                    Transform::from_xyz(0.0, 0.0, -0.1)
                        .with_scale(Vec3::splat(radius - shapes::inset(shape, RING_WIDTH))),
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
                    GraphText { size: label_size },
                    Text2d::new(note.name.clone()),
                    font("Geist, sans-serif", label_size, weight),
                    TextColor(color::MUTED_FOREGROUND),
                    Transform::from_xyz(0.0, -radius * reach(shape) - 12.0, 0.0),
                    RenderLayers::layer(LAYER),
                ));
                spawn_tick(node, radius, &state);
                if layout.parent.contains(&Some(slot)) {
                    fold::spawn_badge(node, radius, &state);
                }
            });
    }
    let mut edges = Vec::new();
    for &id in &layout.notes {
        for &target in &vault.notes[id].outgoing {
            if let (Some(a), Some(b)) = (entity_of(id), entity_of(target)) {
                edges.push((a, b));
            }
        }
    }
    let place = |id: NoteId| {
        layout
            .notes
            .iter()
            .position(|&n| n == id)
            .map(|s| layout.homes[s])
    };
    state.roads = routes::roads(&vault, &place, layout.notes[layout.root])
        .into_iter()
        .filter_map(|(a, b, uses)| {
            Some(roads::Road {
                a: entity_of(a)?,
                b: entity_of(b)?,
                bend: roads::bend_of(a, b),
                uses,
            })
        })
        .collect();
    state.trails = layout
        .parent
        .iter()
        .enumerate()
        .filter_map(|(child, up)| Some((entities[child], entities[(*up)?])))
        .collect();
    state.edges = edges;
    state.refold = true;
    if first {
        state.framing = true;
    }
}

/// Where notes start when the map is first built, as a share of the way to their homes.
const START_SPREAD: f32 = 0.1;

/// The notes on the map and where they sit.
struct Layout {
    /// The notes, in vault order: every note but the reading paths, the map itself and the
    /// app's own notes.
    notes: Vec<NoteId>,
    /// Home of each (as the map says, or beside the note it hangs from), its rank and the
    /// note it hangs from (indices into `notes`).
    homes: Vec<Vec2>,
    rank: Vec<usize>,
    parent: Vec<Option<usize>>,
    /// The hub, the root of the tree.
    root: usize,
}

fn home_positions(vault: &VaultData) -> Layout {
    let notes: Vec<NoteId> = (0..vault.notes.len())
        .filter(|&i| !vault.is_meta(i) && !tree::is_app_note(vault, i))
        .collect();
    let mut rank: Vec<usize> = notes.iter().map(|&id| tree::rank(vault, id)).collect();
    let root = rank.iter().position(|&r| r == 0).unwrap_or_else(|| {
        (0..notes.len())
            .max_by_key(|&n| {
                vault.notes[notes[n]].outgoing.len() + vault.notes[notes[n]].backlinks.len()
            })
            .unwrap_or(0)
    });
    let parent = if notes.is_empty() {
        Vec::new()
    } else {
        tree::spanning_tree(vault, &notes, &rank, root)
    };
    for (n, r) in rank.iter_mut().enumerate() {
        if n == root {
            *r = 0;
        } else if *r == 0 {
            // A second hub is just an area.
            *r = 1;
        }
    }
    let placed: Vec<Option<Vec2>> = notes
        .iter()
        .enumerate()
        .map(|(n, &id)| {
            let at = vault
                .map
                .as_ref()?
                .place(&vault.notes[id].name)
                .map(|p| p.at);
            if n == root {
                Some(at.unwrap_or_default())
            } else {
                at
            }
        })
        .collect();
    let homes = tree::place_the_rest(&placed, &parent);
    Layout {
        notes,
        homes,
        rank,
        parent,
        root,
    }
}

/// Spawns the tick shown on a visited note: a small green disc low on its right.
fn spawn_tick(node: &mut ChildSpawnerCommands, radius: f32, state: &GraphState) {
    let font = TextFont {
        font: bevy::text::FontSource::families("lucide"),
        font_size: TICK_ICON.into(),
        ..default()
    };
    let check = bevsidian_core::icons::find("check").unwrap_or('✓');
    node.spawn((
        GraphVisited,
        Transform::from_translation(Vec2::new(radius * 0.72, -radius * 0.72).extend(0.25)),
        Visibility::Hidden,
        RenderLayers::layer(LAYER),
    ))
    .with_children(|tick| {
        tick.spawn((
            Mesh2d(state.circle.clone()),
            MeshMaterial2d(state.paint.ring.clone()),
            Transform::from_scale(Vec3::splat(TICK_RADIUS + 1.5)),
            RenderLayers::layer(LAYER),
        ));
        tick.spawn((
            Mesh2d(state.circle.clone()),
            MeshMaterial2d(state.paint.visited.clone()),
            Transform::from_xyz(0.0, 0.0, 0.01).with_scale(Vec3::splat(TICK_RADIUS)),
            RenderLayers::layer(LAYER),
        ));
        tick.spawn((
            GraphText { size: TICK_ICON },
            Text2d::new(check.to_string()),
            font,
            LineHeight::RelativeToFont(1.0),
            TextColor(color::BACKGROUND),
            Transform::from_xyz(0.0, 0.0, 0.02),
            RenderLayers::layer(LAYER),
        ));
    });
}

/// Size of the visited tick and its glyph (world units).
const TICK_RADIUS: f32 = 7.0;
const TICK_ICON: f32 = 10.0;

/// Icon colours.
const NOTE_COLOR: Color = color::BODY;
const HOVERED_COLOR: Color = color::FOREGROUND;
/// The outline of a shape: the border token over the page, made opaque.
const RING_COLOR: Color = Color::srgb(0.24, 0.24, 0.24);
/// Width of the outline, in world units.
const RING_WIDTH: f32 = 1.5;
/// How far the halo of the current note reaches past its shape.
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
        image.resize(Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        });
    }
}

/// How fast (per second) notes float home, and into their parent when folding.
const ARRIVE_EASE: f32 = 4.0;
const TUCK_EASE: f32 = 7.0;

/// Moves the notes that are floating: home (the map just opened, or they unfolded) or into
/// the note they fold into. Every other note stays where the map puts it; a dragged one (only
/// while editing) follows the pointer.
fn motion(
    time: Res<Time>,
    mut state: ResMut<GraphState>,
    mut nodes: Query<(Entity, &mut GraphNode, &mut Transform)>,
) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    let ease = |rate: f32| 1.0 - (-rate * dt).exp();
    let mut moving = false;
    for (entity, mut node, mut transform) in &mut nodes {
        if node.hidden || state.dragging == Some(entity) {
            continue;
        }
        let at = transform.translation.truncate();
        let next = if node.tucking {
            moving = true;
            at.lerp(node.tuck_to, ease(TUCK_EASE))
        } else if node.arriving {
            let next = at.lerp(node.home, ease(ARRIVE_EASE));
            node.arriving = next.distance(node.home) > 0.5;
            moving |= node.arriving;
            if node.arriving { next } else { node.home }
        } else {
            continue;
        };
        transform.translation.x = next.x;
        transform.translation.y = next.y;
    }
    state.moving = moving;
}

/// Keeps the shown notes (or the followed route) in frame while they settle.
fn frame(
    mut state: ResMut<GraphState>,
    nodes: Query<(&Transform, &GraphNode), Without<GraphCamera>>,
    viewport: Query<&ComputedNode, With<GraphViewport>>,
    nav: Res<Nav>,
) {
    if !state.framing || nav.view.get_untracked() != View::Graph {
        return;
    }
    let Ok(viewport) = viewport.single() else {
        return;
    };
    let route = state.frame_route;
    state.camera_goal = Some(fit(
        nodes.iter().filter(|(_, n)| !route || n.step.is_some()),
        viewport,
    ));
    if !state.moving {
        state.framing = false;
    }
}

/// Label font size at zoom 1.
const LABEL_SIZE: f32 = 16.0;
/// Smallest a note's name, and a town's, is drawn on screen (in pixels): smaller, it hides.
const NAMES_PX: f32 = 7.0;
const TOWN_NAMES_PX: f32 = 4.0;
/// Smallest a note's shape is drawn on screen (its radius in pixels): the hub, a town, others.
const HUB_PX: f32 = 22.0;
const TOWN_PX: f32 = 14.0;
const NOTE_PX: f32 = 10.0;

/// Re-rasterizes text for the current zoom so it stays sharp when zoomed in. The size moves
/// in steps of √2 (so a smooth zoom does not re-render text every frame) and the text's scale
/// compensates, keeping its size in the world unchanged.
fn sharpen_labels(
    camera: Query<&Projection, With<GraphCamera>>,
    mut texts: Query<(Ref<GraphText>, &mut TextFont, &mut Transform)>,
    mut current: Local<Option<i32>>,
) {
    let Ok(projection) = camera.single() else {
        return;
    };
    let zoom = 1.0 / ortho_scale(projection);
    let step = (zoom.log2() * 2.0).ceil().max(0.0) as i32;
    let changed = *current != Some(step);
    *current = Some(step);
    let factor = (step as f32 / 2.0).exp2();
    for (text, mut font, mut transform) in &mut texts {
        // New text (the map was rebuilt) starts at its plain size.
        if changed || text.is_added() {
            font.font_size = (text.size * factor).into();
            transform.scale = Vec3::splat(1.0 / factor);
        }
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

#[allow(clippy::type_complexity)]
fn style_nodes(
    nav: Res<Nav>,
    state: Res<GraphState>,
    mut nodes: Query<(Entity, &GraphNode, &Children, &mut Transform)>,
    mut icons: Query<&mut TextColor, (With<GraphIcon>, Without<GraphLabel>)>,
    mut labels: Query<&mut TextColor, (With<GraphLabel>, Without<GraphIcon>)>,
    mut fills: Query<&mut MeshMaterial2d<ColorMaterial>, (With<GraphFill>, Without<GraphRing>)>,
    mut rings: Query<
        (&mut MeshMaterial2d<ColorMaterial>, &mut Transform),
        (With<GraphRing>, Without<GraphFill>, Without<GraphNode>),
    >,
    mut ticks: Query<&mut Visibility, (With<GraphVisited>, Without<GraphLabel>)>,
    mut label_shown: Query<&mut Visibility, (With<GraphLabel>, Without<GraphVisited>)>,
    camera: Query<&Projection, With<GraphCamera>>,
) {
    if nav.view.get_untracked() != View::Graph {
        return;
    }
    // Zoomed out, only the names of the bigger places show, like on a map.
    let scale = camera.single().map_or(1.0, ortho_scale);
    let current = nav.current.get_untracked();
    let visited = nav.visited.get_untracked();
    let focus = state.hovered.or(state.dragging);
    let focus_neighbors = focus.map(|f| neighbors(&state, f)).unwrap_or_default();
    let paint = &state.paint;
    for (entity, node, children, mut transform) in &mut nodes {
        // Zoomed out, notes keep a size on screen (bigger places bigger), like map markers.
        let least = match node.rank {
            0 => HUB_PX,
            1 => TOWN_PX,
            _ => NOTE_PX,
        };
        let grow = Vec3::splat((scale * least / node.radius).max(1.0));
        if transform.scale != grow {
            transform.scale = grow;
        }
        let lit = focus.is_none_or(|f| f == entity || focus_neighbors.contains(&entity));
        let is_current = node.note == current;
        let selected = state.selected == Some(entity);
        let icon = if is_current {
            color::BACKGROUND
        } else if Some(entity) == focus {
            HOVERED_COLOR
        } else if !lit {
            color::DISABLED_FOREGROUND
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
        let (ring, ring_radius) = if selected {
            (&paint.selected_ring, node.radius + 2.0)
        } else if is_current {
            (&paint.current_ring, node.radius + HALO)
        } else if Some(entity) == focus || node.rank == 0 {
            (&paint.ring_hovered, node.radius)
        } else {
            (&paint.ring, node.radius)
        };
        let fill = if is_current {
            &paint.current_fill
        } else {
            &paint.fill
        };
        let on_screen = LABEL_SIZE / scale;
        let named = is_current
            || Some(entity) == focus
            || node.step.is_some()
            || match node.rank {
                0 => true,
                1 => on_screen >= TOWN_NAMES_PX,
                _ => on_screen >= NAMES_PX,
            };
        for &child in children {
            if let Ok(mut visibility) = label_shown.get_mut(child) {
                visibility.set_if_neq(if named {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
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
            } else if let Ok(mut visibility) = ticks.get_mut(child) {
                let shows = !is_current && visited.contains(&node.note);
                visibility.set_if_neq(if shows {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
        }
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
    // Pointer in UI physical space, then relative to the viewport centre.
    let local = transform
        .try_inverse()?
        .transform_point2(pointer / node.inverse_scale_factor);
    // The render target is sized in physical px of the node, and the camera maps one target
    // pixel to one world unit at scale 1.
    let world = Vec2::new(local.x, -local.y) * ortho_scale(projection);
    Some(cam_transform.translation().truncate() + world)
}

fn pick(world: Vec2, nodes: &Query<(Entity, &Transform, &GraphNode)>) -> Option<Entity> {
    nodes
        .iter()
        .filter(|(_, t, n)| {
            !n.hidden
                && !n.tucking
                && t.translation.truncate().distance(world) <= n.radius * t.scale.x + 12.0
        })
        .min_by(|a, b| {
            let da = a.1.translation.truncate().distance(world);
            let db = b.1.translation.truncate().distance(world);
            da.total_cmp(&db)
        })
        .map(|(e, ..)| e)
}

/// What is under the pointer: a fold badge, a note, or (while editing) a map object.
fn pick_any(
    world: Vec2,
    editing: bool,
    nodes: &Query<(Entity, &Transform, &GraphNode)>,
    things: &Query<(Entity, &GlobalTransform, &world::MapThing)>,
) -> Option<Entity> {
    let badge = if editing {
        None
    } else {
        fold::badge_at(world, nodes)
    };
    badge
        .or_else(|| pick(world, nodes))
        .or_else(|| editing.then(|| edit::thing_at(world, things)).flatten())
}

#[allow(clippy::too_many_arguments)]
fn on_move(
    event: On<PointerMove>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<GraphCamera>>,
    nodes: Query<(Entity, &Transform, &GraphNode)>,
    things: Query<(Entity, &GlobalTransform, &world::MapThing)>,
    nav: Res<Nav>,
    mut state: ResMut<GraphState>,
    mut cursor: Query<&mut bevy::picking::cursor::EntityCursor>,
) {
    if state.dragging.is_some() {
        return;
    }
    let Some(world) = to_world(event.pointer.position, event.entity, &nodes_ui, &camera) else {
        return;
    };
    let editing = nav.map_edit.get_untracked();
    let hovered = pick_any(world, editing, &nodes, &things);
    if state.hovered != hovered {
        state.hovered = hovered;
    }
    let icon = match (hovered, editing) {
        (Some(_), true) => bevy::window::SystemCursorIcon::Move,
        (Some(_), false) => bevy::window::SystemCursorIcon::Pointer,
        (None, _) => bevy::window::SystemCursorIcon::Grab,
    };
    let wanted = bevy::picking::cursor::EntityCursor::System(icon);
    if let Ok(mut c) = cursor.get_mut(event.entity)
        && *c != wanted
    {
        *c = wanted;
    }
}

fn on_out(_: On<PointerOut>, mut state: ResMut<GraphState>) {
    if state.dragging.is_none() {
        state.hovered = None;
    }
}

#[allow(clippy::too_many_arguments)]
fn on_press(
    event: On<PointerPress>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<GraphCamera>>,
    nodes: Query<(Entity, &Transform, &GraphNode)>,
    things: Query<(Entity, &GlobalTransform, &world::MapThing)>,
    nav: Res<Nav>,
    mut state: ResMut<GraphState>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    // Touch has no hover, so pick here as well.
    let editing = nav.map_edit.get_untracked();
    let node = to_world(event.pointer.position, event.entity, &nodes_ui, &camera)
        .and_then(|world| pick_any(world, editing, &nodes, &things));
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
    mut state: ResMut<GraphState>,
    nodes: Query<&GraphNode>,
    nav: Res<Nav>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    // Only a press that stayed put is a click; dragging or panning never opens a note.
    let Some(Press {
        node: target,
        moved: false,
    }) = state.press
    else {
        return;
    };
    if nav.map_edit.get_untracked() {
        // Editing: a click chooses what the inspector edits (or, on the sea, nothing).
        state.selected = target;
        return;
    }
    let Some(entity) = target else {
        return;
    };
    let Ok(node) = nodes.get(entity) else {
        return;
    };
    // A note holding folded notes folds and unfolds on a click and opens on a double click
    // (which puts back what its first click folded); any other note opens on a click.
    let foldable = node.folds > 0;
    let double = event.count >= 2 && state.last_click == Some(entity);
    state.last_click = Some(entity);
    if foldable {
        state.fold_toggle = Some(entity);
    }
    if !foldable || double {
        nav.open(node.note, None);
    }
}

#[allow(clippy::too_many_arguments)]
fn on_drag(
    event: On<PointerDrag>,
    time: Res<Time<Real>>,
    touches: Res<Touches>,
    nav: Res<Nav>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut camera: Query<(&Camera, &mut Transform, &Projection), With<GraphCamera>>,
    mut moved: Query<
        &mut Transform,
        (
            Or<(With<GraphNode>, With<world::MapThing>)>,
            Without<GraphCamera>,
        ),
    >,
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
        // Only an edited map moves; otherwise a drag anywhere pans.
        if nav.map_edit.get_untracked() {
            state.dragging = press.node;
            state.selected = press.node.or(state.selected);
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
        if let Ok(mut transform) = moved.get_mut(entity) {
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
    // A fling keeps panning (see `camera_motion`); a dragged note or object stays where it
    // was dropped, and is saved into the map (see `edit`).
    let dropped = state.dragging.take();
    if dropped.is_some() || !state.press.is_some_and(|p| p.moved) {
        state.pan_velocity = Vec2::ZERO;
    }
    state.dropped = dropped;
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
const MAX_SCALE: f32 = 9.0;
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
        let t = 1.0 - (-10.0 * dt).exp();
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

/// Camera position and zoom that show these notes (at their homes: where they are going).
fn fit<'a>(
    nodes: impl Iterator<Item = (&'a Transform, &'a GraphNode)>,
    viewport: &ComputedNode,
) -> (Vec2, f32) {
    let mut bounds: Option<Rect> = None;
    for (_, node) in nodes.filter(|(_, n)| !n.hidden && !n.tucking) {
        // Leave room for the note and its label below.
        let r = Rect::from_center_half_size(node.home, Vec2::new(90.0, 60.0));
        bounds = Some(bounds.map_or(r, |b| b.union(r)));
    }
    let Some(bounds) = bounds else {
        return (Vec2::ZERO, 1.0);
    };
    let size = viewport.size().max(Vec2::ONE);
    let scale = (bounds.size() / size).max_element() * 1.1;
    // A few notes are framed at a comfortable size, not blown up to fill the view.
    (bounds.center(), scale.clamp(FIT_MIN_SCALE, MAX_SCALE))
}

#[cfg(test)]
mod tests;
