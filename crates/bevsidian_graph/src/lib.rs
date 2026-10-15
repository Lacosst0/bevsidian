//! The graph view, drawn as a hand-made star map.
//!
//! Every note is a filled shape where the vault's map note puts it ([`bevsidian_core::map`]):
//! Bevy, the largest, in the middle, the main quest drawn round it as a bird after Bevy's
//! logo, and the sky behind them ([`world`]). The shape says what a note is (a hexagon for an area, a
//! diamond for a milestone, a square for a crate, a star for Bevy…; see [`shapes`]), the
//! note's icon sits inside it and its name above; it is filled grey once the note is read. Routes are coloured lines with arrows in reading order, their first page filled
//! with their colour ([`lines`], [`routes`]): the map shows the main quest, the routes the reader chose
//! in its checklist and the route being followed; everything else shows with the whole map
//! ([`show`]).
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
mod hint;
mod lines;
mod mesh;
mod routes;
mod shapes;
mod show;
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
                show::show_nodes,
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
                limit_zoom,
                frame,
                pinch_zoom,
                camera_motion,
                sharpen_labels,
                style_nodes,
                declutter_labels,
                fit_plates,
                world::scale_objects,
                world::parallax,
                world::draw_sky,
                world::draw_field,
                lines::draw_lines,
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

/// A note on the map, drawn as its icon inside its shape, with its name above.
#[derive(Component)]
#[allow(clippy::struct_excessive_bools)] // Independent states of one note, not a mode.
struct GraphNode {
    note: NoteId,
    /// Its size: the radius of its shape.
    radius: f32,
    /// How deep it sits in the tree (see [`tree::rank`]); 0 for the hub.
    rank: usize,
    /// On a route the map shows (or the hub): always on the map.
    forced: bool,
    /// Its place on the followed route (from 0), if it is on it.
    step: Option<usize>,
    /// Not drawn or picked: off the routes shown.
    hidden: bool,
    /// Its place on the map.
    home: Vec2,
    /// Floating to its home (the map just opened, or it appeared).
    arriving: bool,
}

/// A note's name, above its shape, over a dark plate ([`GraphPlate`]) that keeps lines from
/// showing between its letters. Zoomed out, a name is drawn larger than its size in the world
/// so it stays readable.
#[derive(Component)]
struct GraphLabel {
    /// Height of the name's bottom above the note's centre (world units, at the note's scale).
    lift: f32,
    /// How much larger than its size in the world the name is drawn.
    boost: f32,
}

#[derive(Component)]
struct GraphPlate;

/// The note's icon, inside its shape.
#[derive(Component)]
struct GraphIcon;

/// The inside of a note's shape.
#[derive(Component)]
struct GraphFill;

/// The outline of a note's shape: hairline, or bolder in a route's colour.
#[derive(Component)]
struct GraphRing {
    thin: Handle<Mesh>,
    bold: Handle<Mesh>,
}

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
    /// The lines of every route.
    lines: Vec<lines::Line>,
    /// The colour of each route's lines.
    colors: HashMap<NoteId, Color>,
    /// The routes (but the main quest) each note is a step of, in vault order, or is tied to.
    note_routes: HashMap<NoteId, Vec<NoteId>>,
    /// Where each route begins (its first page off the main quest): the routes from it.
    stations: HashMap<NoteId, Vec<NoteId>>,
    /// The notes no route takes tied into each route.
    tied: HashMap<NoteId, Vec<NoteId>>,
    /// The routes shown (their path notes), the one followed, and whether the whole map is.
    active: Vec<NoteId>,
    followed: Option<NoteId>,
    whole: bool,
    /// The map is being edited (every note and line shows).
    editing: bool,
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
    /// Farthest the camera zooms out (its scale): the notes shown with room around them,
    /// worked out by [`limit_zoom`] (0 until then).
    farthest: f32,
    /// Distance between two fingers in the previous frame of a pinch.
    pinch: Option<f32>,
    /// Some note is still floating.
    moving: bool,
    /// While editing: the note or object chosen, and the one just dropped (to save).
    selected: Option<Entity>,
    dropped: Option<Entity>,
    /// A change asked for in the edit bar.
    edit_request: Option<edit::EditRequest>,
    /// The map changed: redraw the sky.
    sky_dirty: bool,
    /// How far the background stars reach from the capital.
    field_reach: f32,
    /// Shared meshes and materials.
    circle: Handle<Mesh>,
    meshes: NodeMeshes,
    paint: NodePaint,
}

/// Materials of a note's shape and icon, per state: a note the reader has not opened yet
/// is dark inside, a read one filled grey, the one being read filled orange.
#[derive(Default)]
struct NodePaint {
    /// The inside of a note's shape.
    fill: Handle<ColorMaterial>,
    fill_current: Handle<ColorMaterial>,
    fill_read: Handle<ColorMaterial>,
    ring: Handle<ColorMaterial>,
    ring_lit: Handle<ColorMaterial>,
    ring_hovered: Handle<ColorMaterial>,
    ring_current: Handle<ColorMaterial>,
    plate: Handle<ColorMaterial>,
    /// Each route's colour, for its notes' outlines and its first page, and dimmed (with the
    /// whole map, for the routes not shown).
    routes: HashMap<NoteId, Handle<ColorMaterial>>,
    routes_dim: HashMap<NoteId, Handle<ColorMaterial>>,
}

/// Shared meshes of the notes.
#[derive(Default)]
struct NodeMeshes {
    fills: HashMap<Shape, Handle<Mesh>>,
    rings: HashMap<Shape, Handle<Mesh>>,
    bold_rings: HashMap<Shape, Handle<Mesh>>,
    plate: Handle<Mesh>,
}

impl GraphState {
    /// How far out the camera may zoom (its largest scale).
    fn max_scale(&self) -> f32 {
        if self.farthest > 0.0 {
            self.farthest
        } else {
            MAX_SCALE
        }
    }

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

/// The colour of a route's lines: the main quest's are white; another route takes the
/// colour its `color:` property names, or else one picked by where it sits round the map
/// ([`color::map_route`]), starting from the main quest's first step and turning
/// anticlockwise. Where a route sits is the middle of its steps off the main quest.
pub fn route_color(vault: &VaultData, route: NoteId) -> Color {
    let path = vault.paths.iter().find(|p| p.note == route);
    if let Some(named) = path
        .and_then(|p| p.color.as_deref())
        .and_then(color::map_route_named)
    {
        return named;
    }
    let wheel = route_wheel(vault);
    wheel
        .iter()
        .position(|&r| r == route)
        .map_or(color::MAP_LINE, |i| color::map_route(i, wheel.len()))
}

/// The routes but the main quest, in the order they sit round the map (see [`route_color`]).
fn route_wheel(vault: &VaultData) -> Vec<NoteId> {
    use bevsidian_core::vault::Quest;
    let place = |id: NoteId| {
        vault
            .map
            .as_ref()?
            .place(&vault.notes[id].name)
            .map(|p| p.at)
    };
    let main: &[NoteId] = vault
        .paths
        .iter()
        .find(|p| p.quest == Quest::Main)
        .map_or(&[], |p| p.steps.as_slice());
    let from = main
        .first()
        .and_then(|&s| place(s))
        .and_then(Vec2::try_normalize)
        .unwrap_or(Vec2::NEG_Y);
    let mut routes: Vec<(f32, NoteId)> = vault
        .paths
        .iter()
        .filter(|p| p.quest != Quest::Main)
        .map(|p| {
            let middle: Vec2 = p
                .steps
                .iter()
                .filter(|s| !main.contains(s))
                .filter_map(|&s| place(s))
                .sum();
            // The turn from the main quest's start, anticlockwise, in 0..TAU.
            let turn = from.angle_to(middle).rem_euclid(std::f32::consts::TAU);
            (if middle == Vec2::ZERO { 0.0 } else { turn }, p.note)
        })
        .collect();
    routes.sort_by(|a, b| a.0.total_cmp(&b.0));
    routes.into_iter().map(|(_, r)| r).collect()
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
    // Opaque meshes let depth decide what covers what (the sky under the lines, the lines
    // under the notes and their names).
    let opaque = |materials: &mut Assets<ColorMaterial>| {
        MeshMaterial2d(materials.add(ColorMaterial {
            alpha_mode: bevy::sprite_render::AlphaMode2d::Opaque,
            ..default()
        }))
    };
    commands.spawn((
        lines::LinesMesh,
        Mesh2d(meshes.add(mesh::empty())),
        opaque(&mut materials),
        Transform::default(),
        RenderLayers::layer(LAYER),
    ));
    commands.spawn((
        world::Grid,
        Mesh2d(meshes.add(mesh::empty())),
        opaque(&mut materials),
        Transform::default(),
        RenderLayers::layer(LAYER),
    ));
    commands.spawn((
        world::Starfield,
        Mesh2d(meshes.add(mesh::empty())),
        opaque(&mut materials),
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

    state.paint = NodePaint {
        fill: materials.add(color::CARD),
        fill_current: materials.add(color::LINK),
        fill_read: materials.add(color::MAP_FILL_READ),
        ring: materials.add(color::MAP_RING),
        ring_lit: materials.add(color::MAP_RING_LIT),
        ring_hovered: materials.add(color::MUTED_FOREGROUND),
        ring_current: materials.add(color::LINK),
        plate: materials.add(color::MAP_PLATE),
        routes: HashMap::default(),
        routes_dim: HashMap::default(),
    };
    state.circle = meshes.add(Circle::new(1.0));
    state.meshes = NodeMeshes {
        fills: Shape::ALL
            .into_iter()
            .map(|shape| (shape, meshes.add(shapes::fill(shape))))
            .collect(),
        rings: Shape::ALL
            .into_iter()
            .map(|shape| (shape, meshes.add(shapes::ring(shape, RING_WIDTH))))
            .collect(),
        bold_rings: Shape::ALL
            .into_iter()
            .map(|shape| (shape, meshes.add(shapes::ring(shape, BOLD_RING_WIDTH))))
            .collect(),
        plate: meshes.add(Rectangle::new(1.0, 1.0)),
    };
}

/// The shape a note's rank gives it when the map names none.
fn default_shape(rank: usize) -> Shape {
    match rank {
        0 => Shape::Star,
        1 => Shape::Hexagon,
        4 => Shape::Square,
        _ => Shape::Circle,
    }
}

/// How important a note is on the map, which decides how big it and its name are drawn:
/// Bevy, then the main quest and the areas, lessons, finer points (sub-concepts and crates
/// compared), and crates the smallest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tier {
    Hub,
    Main,
    Area,
    Lesson,
    Detail,
    Crate,
}

impl Tier {
    fn of(rank: usize, shape: Shape, on_main_quest: bool) -> Tier {
        match rank {
            0 => Tier::Hub,
            _ if on_main_quest => Tier::Main,
            1 => Tier::Area,
            4 => Tier::Crate,
            _ if shape == Shape::Square => Tier::Crate,
            3 => Tier::Detail,
            _ if shape == Shape::Pentagon => Tier::Detail,
            _ => Tier::Lesson,
        }
    }

    /// The note's radius and its name's size (world units), before the map's size.
    fn sizes(self) -> (f32, f32) {
        match self {
            Tier::Hub => (46.0, 30.0),
            Tier::Area => (34.0, 26.0),
            Tier::Main => (31.0, 24.0),
            Tier::Lesson => (25.0, 20.0),
            Tier::Detail => (20.0, 17.0),
            Tier::Crate => (15.0, 14.0),
        }
    }

    fn weight(self) -> FontWeight {
        match self {
            Tier::Hub | Tier::Area | Tier::Main => FontWeight::SEMIBOLD,
            _ => FontWeight::MEDIUM,
        }
    }
}

/// The icon's size inside a shape, as a share of its radius (a star's points leave less room).
fn icon_share(shape: Shape) -> f32 {
    match shape {
        Shape::Star => 0.62,
        Shape::Diamond | Shape::Square => 0.8,
        _ => 0.9,
    }
}

/// (Re)builds the nodes and lines from the vault, at startup and after a note
/// (or the map) is edited.
fn spawn_nodes(
    mut commands: Commands,
    vault: Res<Vault>,
    mut state: ResMut<GraphState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    old: Query<Entity, With<GraphNode>>,
    mut built: Local<Option<u64>>,
) {
    let vault = vault.0.clone();
    if *built == Some(vault.revision) {
        return;
    }
    let first = built.is_none();
    *built = Some(vault.revision);
    for entity in &old {
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
    let entities: Vec<Entity> = layout
        .notes
        .iter()
        .map(|_| commands.spawn_empty().id())
        .collect();
    let entity_of = |id: NoteId| layout.slot(id).map(|s| entities[s]);
    let main_steps: Vec<NoteId> = vault
        .paths
        .iter()
        .filter(|p| p.quest == bevsidian_core::vault::Quest::Main)
        .flat_map(|p| p.steps.iter().copied())
        .collect();
    for (slot, &id) in layout.notes.iter().enumerate() {
        let note = &vault.notes[id];
        let rank = layout.rank[slot];
        let place = vault.map.as_ref().and_then(|m| m.place(&note.name));
        let shape = place
            .and_then(|p| p.shape)
            .unwrap_or_else(|| default_shape(rank));
        let tier = Tier::of(rank, shape, main_steps.contains(&id));
        let (base, label_size) = tier.sizes();
        let radius = base * place.map_or(Scale::Normal, |p| p.scale).factor();
        let weight = tier.weight();
        let home = layout.homes[slot];
        let icon_size = (radius * icon_share(shape)).round();
        let fill = state.meshes.fills.get(&shape).cloned().unwrap_or_default();
        let ring = state.meshes.rings.get(&shape).cloned().unwrap_or_default();
        let bold = state
            .meshes
            .bold_rings
            .get(&shape)
            .cloned()
            .unwrap_or_default();
        let (backing, fill_paint, ring_paint, plate_paint) = (
            state.meshes.plate.clone(),
            state.paint.fill.clone(),
            state.paint.ring.clone(),
            state.paint.plate.clone(),
        );
        commands
            .entity(entities[slot])
            .insert((
                GraphNode {
                    note: id,
                    radius,
                    rank,
                    forced: rank == 0,
                    step: None,
                    hidden: false,
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
                    GraphRing {
                        thin: ring.clone(),
                        bold,
                    },
                    Mesh2d(ring),
                    MeshMaterial2d(ring_paint),
                    Transform::from_xyz(0.0, 0.0, -0.1).with_scale(Vec3::splat(radius)),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphFill,
                    Mesh2d(fill),
                    MeshMaterial2d(fill_paint),
                    // Under the outline, which may be bold.
                    Transform::from_xyz(0.0, 0.0, -0.2)
                        .with_scale(Vec3::splat(radius * (1.0 - RING_WIDTH / 2.0))),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphIcon,
                    GraphText { size: icon_size },
                    Text2d::new(note.icon.to_string()),
                    font("lucide", icon_size, FontWeight::NORMAL),
                    // One-em line: Lucide icons are centred in their em box.
                    LineHeight::RelativeToFont(1.0),
                    TextColor(color::MUTED_FOREGROUND),
                    Transform::default(),
                    RenderLayers::layer(LAYER),
                ));
                let lift = radius * shapes::top(shape) + LABEL_GAP;
                node.spawn((
                    GraphLabel { lift, boost: 1.0 },
                    GraphText { size: label_size },
                    Text2d::new(label_text(&note.name, None)),
                    font("Geist, sans-serif", label_size, weight),
                    TextColor(color::MUTED_FOREGROUND),
                    Transform::from_xyz(0.0, lift + label_size / 2.0, 0.0),
                    RenderLayers::layer(LAYER),
                ))
                .with_children(|label| {
                    label.spawn((
                        GraphPlate,
                        Mesh2d(backing),
                        MeshMaterial2d(plate_paint),
                        Transform::from_xyz(0.0, 0.0, -0.05).with_scale(Vec3::ZERO),
                        RenderLayers::layer(LAYER),
                    ));
                });
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
    let place = |id: NoteId| layout.slot(id).map(|s| layout.homes[s]);
    let drawn = routes::lines(&vault, &place, layout.notes[layout.root]);
    let starts = &drawn.starts;
    state.lines = drawn
        .lines
        .iter()
        .filter_map(|(a, b, uses)| {
            // The route that leaves the main quest by this line.
            let start = starts
                .iter()
                .find(|&&(_, from, to)| (from, to) == (*a, *b) || (from, to) == (*b, *a))
                .map(|&(route, ..)| route);
            Some(lines::Line {
                a: entity_of(*a)?,
                b: entity_of(*b)?,
                uses: uses.clone(),
                start,
                tie: false,
            })
        })
        .collect();
    state.colors = vault
        .paths
        .iter()
        .map(|p| (p.note, route_color(&vault, p.note)))
        .collect();
    for path in vault
        .paths
        .iter()
        .filter(|p| p.quest != bevsidian_core::vault::Quest::Main)
    {
        let tint = state.colors[&path.note];
        state
            .paint
            .routes
            .entry(path.note)
            .or_insert_with(|| materials.add(tint));
        state
            .paint
            .routes_dim
            .entry(path.note)
            .or_insert_with(|| materials.add(tint.mix(&color::BACKGROUND, lines::WHOLE_DIM)));
    }
    let members = membership(&vault, &place, &layout.notes, &drawn);
    for &(note, to, route) in &members.ties {
        if let (Some(a), Some(b)) = (entity_of(note), entity_of(to)) {
            state.lines.push(lines::Line {
                a,
                b,
                uses: route
                    .map(|route| routes::Use {
                        route,
                        kind: bevsidian_core::vault::StepKind::Core,
                        forward: true,
                    })
                    .into_iter()
                    .collect(),
                start: None,
                tie: true,
            });
        }
    }
    state.note_routes = members.note_routes;
    state.stations = members.stations;
    state.tied = members.tied;
    state.edges = edges;
    if first {
        state.framing = true;
    }
}

/// Which routes each note belongs to, where each route begins, and the ties.
#[derive(Default)]
struct Membership {
    /// The routes (but the main quest) each note is a step of, or is tied to.
    note_routes: HashMap<NoteId, Vec<NoteId>>,
    /// Each route's first page off the main quest: the routes beginning at a note.
    stations: HashMap<NoteId, Vec<NoteId>>,
    /// The notes no route takes tied into each route.
    tied: HashMap<NoteId, Vec<NoteId>>,
    /// Each tie: the note, the note it hangs from, and the route it joins.
    ties: Vec<(NoteId, NoteId, Option<NoteId>)>,
}

/// Works out [`Membership`]. The main quest's notes belong to the main quest only: no
/// other route colours them. A note no route takes is tied to a note near it
/// ([`routes::ties`]) and joins its route.
fn membership(
    vault: &VaultData,
    place: &dyn Fn(NoteId) -> Option<Vec2>,
    notes: &[NoteId],
    drawn: &routes::RouteLines,
) -> Membership {
    use bevsidian_core::vault::Quest;
    let main_steps: Vec<NoteId> = vault
        .paths
        .iter()
        .filter(|p| p.quest == Quest::Main)
        .flat_map(|p| p.steps.iter().copied())
        .collect();
    let mut out = Membership::default();
    for path in vault.paths.iter().filter(|p| p.quest != Quest::Main) {
        for &step in path.steps.iter().filter(|s| !main_steps.contains(s)) {
            let routes = out.note_routes.entry(step).or_default();
            if !routes.contains(&path.note) {
                routes.push(path.note);
            }
        }
        if let Some(&first) = path.steps.iter().find(|s| !main_steps.contains(s)) {
            out.stations.entry(first).or_default().push(path.note);
        }
    }
    let connected = drawn.lines.iter().flat_map(|(a, b, _)| [*a, *b]).collect();
    for (note, to) in routes::ties(vault, place, notes, &connected) {
        let route = out.note_routes.get(&to).and_then(|r| r.first()).copied();
        if let Some(route) = route {
            out.note_routes.insert(note, vec![route]);
            out.tied.entry(route).or_default().push(note);
        }
        out.ties.push((note, to, route));
    }
    out
}

/// The colour of each note's route, as the map shows it (a note on several routes takes the
/// first's); the main quest's notes, and notes on no route, have none.
pub fn note_colors(vault: &VaultData) -> HashMap<NoteId, Color> {
    let layout = home_positions(vault);
    let place = |id: NoteId| layout.slot(id).map(|s| layout.homes[s]);
    let Some(&hub) = layout.notes.get(layout.root) else {
        return HashMap::default();
    };
    let drawn = routes::lines(vault, &place, hub);
    membership(vault, &place, &layout.notes, &drawn)
        .note_routes
        .into_iter()
        .filter_map(|(note, routes)| Some((note, route_color(vault, *routes.first()?))))
        .collect()
}

/// Where notes start when the map is first built, as a share of the way to their homes.
const START_SPREAD: f32 = 0.1;

/// The notes on the map and where they sit.
struct Layout {
    /// The notes, in vault order: every note but the reading paths, the map itself and the
    /// app's own notes.
    notes: Vec<NoteId>,
    /// Home of each (as the map says, or beside the note it hangs from) and its rank.
    homes: Vec<Vec2>,
    rank: Vec<usize>,
    /// The hub, the root of the tree.
    root: usize,
}

impl Layout {
    /// Where a note is in `notes`.
    fn slot(&self, id: NoteId) -> Option<usize> {
        self.notes.iter().position(|&n| n == id)
    }
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
        root,
    }
}

/// A note's name, with its step number on the followed route.
fn label_text(name: &str, step: Option<usize>) -> String {
    match step {
        Some(step) => format!("{}. {name}", step + 1),
        None => name.to_owned(),
    }
}

/// Room around a label on its plate (text units).
const PLATE_PAD: Vec2 = Vec2::new(6.0, 2.0);

/// Fits each label's plate to its text once it is laid out.
fn fit_plates(
    labels: Query<
        (&bevy::text::TextLayoutInfo, &Children),
        (With<GraphLabel>, Changed<bevy::text::TextLayoutInfo>),
    >,
    mut plates: Query<&mut Transform, With<GraphPlate>>,
) {
    for (layout, children) in &labels {
        for &child in children {
            if let Ok(mut plate) = plates.get_mut(child) {
                plate.scale = (layout.size + PLATE_PAD * 2.0 * layout.scale_factor).extend(1.0);
            }
        }
    }
}

/// Room between a note's shape and its label above (world units).
const LABEL_GAP: f32 = 8.0;
/// How far past a note's outline the pointer still picks it (world units).
const PICK_SLOP: f32 = 12.0;
/// Width of a shape's outline, as a share of its radius.
const RING_WIDTH: f32 = 0.08;
const BOLD_RING_WIDTH: f32 = 0.16;

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
    let active = nav.view == View::Graph;
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

/// How fast (per second) notes float home.
const ARRIVE_EASE: f32 = 5.0;

/// Moves the notes that are floating home (the map just opened, or they appeared). Every
/// other note stays where the map puts it; a dragged one (only while editing) follows the
/// pointer.
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
        let next = if node.arriving {
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
    if !state.framing || nav.view != View::Graph {
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

/// A lesson's name size at zoom 1 (others by their [`Tier`]), for when names show.
const LABEL_SIZE: f32 = 21.0;
/// Smallest a note's name, and an area's, is drawn on screen (in pixels): smaller, it hides.
const NAMES_PX: f32 = 8.0;
const TOWN_NAMES_PX: f32 = 4.0;
/// Names smaller than this on screen (in pixels) are drawn up to it, at most so many times
/// their size: an area's name, which shows furthest out, and others.
const READABLE_PX: f32 = 13.0;
const TOWN_BOOST: f32 = 3.0;
const MAX_BOOST: f32 = 1.8;
/// Zoomed out, notes stop shrinking at this many screen pixels per world unit of their
/// radius: every note by the same factor, so the important ones stay the bigger ones (Bevy
/// keeps about 20 px, a crate about 6).
const LEAST_PX_PER_UNIT: f32 = 0.42;

/// Re-rasterizes text for the current zoom so it stays sharp when zoomed in. The size moves
/// in steps of √2 (so a smooth zoom does not re-render text every frame) and the text's scale
/// compensates, keeping its size in the world unchanged (times a name's boost).
fn sharpen_labels(
    camera: Query<&Projection, With<GraphCamera>>,
    mut texts: Query<(
        Ref<GraphText>,
        Option<Ref<GraphLabel>>,
        &mut TextFont,
        &mut Transform,
    )>,
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
    for (text, label, mut font, mut transform) in &mut texts {
        let boosted = label.as_ref().is_some_and(Ref::is_changed);
        // New text (the map was rebuilt) starts at its plain size.
        if changed || text.is_added() || boosted {
            let boost = label.map_or(1.0, |l| l.boost);
            font.font_size = (text.size * boost * factor).into();
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

/// Styles the notes: a note not read yet is dark inside (a grey icon), a read one is filled
/// grey, the one being read orange, the first page of a route with its colour; a
/// note on a route wears its colour on its outline; a hovered note lights up with its neighbours (the
/// rest fade). Zoomed out, notes keep a size on screen and only the bigger places keep their
/// names.
#[allow(clippy::type_complexity)]
fn style_nodes(
    nav: Res<Nav>,
    state: Res<GraphState>,
    mut nodes: Query<(Entity, &GraphNode, &Children, &mut Transform), Without<GraphLabel>>,
    mut labels: Query<
        (
            &mut GraphLabel,
            &GraphText,
            &mut Transform,
            &mut TextColor,
            &mut Visibility,
        ),
        (Without<GraphIcon>, Without<GraphNode>),
    >,
    mut icons: Query<&mut TextColor, (With<GraphIcon>, Without<GraphLabel>)>,
    mut rings: Query<
        (&GraphRing, &mut Mesh2d, &mut MeshMaterial2d<ColorMaterial>),
        Without<GraphFill>,
    >,
    mut fills: Query<&mut MeshMaterial2d<ColorMaterial>, (With<GraphFill>, Without<GraphRing>)>,
    camera: Query<&Projection, With<GraphCamera>>,
) {
    if nav.view != View::Graph {
        return;
    }
    let scale = camera.single().map_or(1.0, ortho_scale);
    let current = nav.current;
    let visited = nav.visited.clone();
    let focus = state.hovered.or(state.dragging);
    let focus_neighbors = focus.map(|f| neighbors(&state, f)).unwrap_or_default();
    let paint = &state.paint;
    for (entity, node, children, mut transform) in &mut nodes {
        let size = Vec3::splat((scale * LEAST_PX_PER_UNIT).max(1.0));
        if transform.scale != size {
            transform.scale = size;
        }
        let near = focus.is_none_or(|f| f == entity || focus_neighbors.contains(&entity));
        let is_current = node.note == current;
        let focused = Some(entity) == focus;
        let selected = state.selected == Some(entity);
        let lit = node.rank == 0 || visited.contains(&node.note);
        let (label, icon) = if is_current {
            (color::FOREGROUND, color::BACKGROUND)
        } else if focused {
            (color::FOREGROUND, color::FOREGROUND)
        } else if !near {
            (color::DISABLED_FOREGROUND, color::DISABLED_FOREGROUND)
        } else if lit {
            (color::BODY, color::BODY)
        } else {
            (color::MUTED_FOREGROUND, color::MUTED_FOREGROUND)
        };
        let on_screen = LABEL_SIZE / scale;
        let named = is_current
            || focused
            || selected
            || node.step.is_some()
            || match node.rank {
                0 => true,
                1 => on_screen >= TOWN_NAMES_PX,
                _ => on_screen >= NAMES_PX,
            };
        // The route it is a step of: the followed one, else one shown, else (with the whole
        // map) any, dimmed.
        let routes = state
            .note_routes
            .get(&node.note)
            .map_or(&[][..], Vec::as_slice);
        let colour_of = |routes: &[NoteId]| {
            routes
                .iter()
                .find(|&&r| Some(r) == state.followed)
                .or_else(|| routes.iter().find(|r| state.active.contains(r)))
                .and_then(|r| paint.routes.get(r))
                .or_else(|| {
                    (state.whole || state.editing)
                        .then(|| routes.first().and_then(|r| paint.routes_dim.get(r)))
                        .flatten()
                })
        };
        let route = colour_of(routes);
        // Where a route begins, the note is filled with its colour.
        let station = state
            .stations
            .get(&node.note)
            .and_then(|routes| colour_of(routes));
        let ring = if is_current || selected {
            &paint.ring_current
        } else if focused || node.rank == 0 {
            &paint.ring_hovered
        } else if let Some(route) = route {
            route
        } else if lit {
            &paint.ring_lit
        } else {
            &paint.ring
        };
        let bold = route.is_some() || is_current || selected;
        // Read, a note is filled grey.
        let read = node.rank != 0 && visited.contains(&node.note);
        let fill = if is_current {
            &paint.fill_current
        } else if let Some(station) = station {
            station
        } else if read {
            &paint.fill_read
        } else {
            &paint.fill
        };
        let icon = if is_current {
            icon
        } else if station.is_some() {
            color::BACKGROUND
        } else if read {
            color::FOREGROUND
        } else {
            icon
        };
        let set = |material: &mut MeshMaterial2d<ColorMaterial>, wanted: &Handle<ColorMaterial>| {
            if material.0 != *wanted {
                material.0 = wanted.clone();
            }
        };
        for &child in children {
            if let Ok((mut name, text, mut at, mut color, mut visibility)) = labels.get_mut(child) {
                // Its size on screen, and how much it is drawn larger to be readable.
                let shown = text.size * size.x / scale;
                let most = if node.rank <= 1 {
                    TOWN_BOOST
                } else {
                    MAX_BOOST
                };
                let boost = (READABLE_PX / shown).clamp(1.0, most);
                if (name.boost - boost).abs() > 0.01 {
                    name.boost = boost;
                    at.translation.y = name.lift + text.size * boost / 2.0;
                }
                color.set_if_neq(TextColor(label));
                visibility.set_if_neq(if named {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            } else if let Ok(mut color) = icons.get_mut(child) {
                color.set_if_neq(TextColor(icon));
            } else if let Ok((meshes, mut mesh, mut material)) = rings.get_mut(child) {
                set(&mut material, ring);
                let wanted = if bold { &meshes.bold } else { &meshes.thin };
                if mesh.0 != *wanted {
                    mesh.0 = wanted.clone();
                }
            } else if let Ok(mut material) = fills.get_mut(child) {
                set(&mut material, fill);
            }
        }
    }
}

/// Hides a name that would overlap one shown before it, so names never run into each other
/// (zoomed out, they are drawn larger than their room), nor over another note. Names go in
/// order: the note being read and the hovered or chosen one, the followed route's steps in
/// order (these always show), the notes of the routes shown, then the rest: the hub, areas,
/// lessons and crates.
fn declutter_labels(
    nav: Res<Nav>,
    state: Res<GraphState>,
    nodes: Query<(Entity, &GraphNode, &Children, &GlobalTransform)>,
    mut labels: Query<
        (
            &GlobalTransform,
            &bevy::text::TextLayoutInfo,
            &mut Visibility,
        ),
        With<GraphLabel>,
    >,
) {
    if nav.view != View::Graph {
        return;
    }
    let current = nav.current;
    let first = |e: Entity| Some(e) == state.hovered || Some(e) == state.selected;
    let mut order: Vec<_> = nodes
        .iter()
        .filter(|(_, node, ..)| !node.hidden)
        .filter_map(|(entity, node, children, _)| {
            let label = children.iter().find(|&c| labels.contains(c))?;
            let rank = (
                node.note != current && !first(entity),
                node.step.unwrap_or(usize::MAX),
                !node.forced,
                node.rank,
            );
            Some((rank, label))
        })
        .collect();
    order.sort_by_key(|&(rank, _)| rank);
    // The notes' shapes, as room already taken.
    let shapes: Vec<Rect> = nodes
        .iter()
        .filter(|(_, node, ..)| !node.hidden)
        .map(|(_, node, _, at)| {
            let (scale, _, centre) = at.to_scale_rotation_translation();
            // The square inside the shape: its corners are not taken.
            Rect::from_center_half_size(
                centre.truncate(),
                Vec2::splat(node.radius * scale.x * 0.75),
            )
        })
        .collect();
    let mut kept: Vec<Rect> = Vec::with_capacity(order.len());
    for (rank, label) in order {
        let Ok((at, layout, mut visibility)) = labels.get_mut(label) else {
            continue;
        };
        if *visibility == Visibility::Hidden {
            continue;
        }
        let (scale, _, centre) = at.to_scale_rotation_translation();
        let size = layout.size * scale.truncate();
        let room = Rect::from_center_size(centre.truncate(), size);
        let always = !rank.0 || rank.1 != usize::MAX;
        let crowded = kept.iter().any(|other| !other.intersect(room).is_empty())
            || (!always && shapes.iter().any(|shape| !shape.intersect(room).is_empty()));
        if crowded && !always {
            *visibility = Visibility::Hidden;
        } else {
            kept.push(room);
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
                && t.translation.truncate().distance(world) <= n.radius * t.scale.x + PICK_SLOP
        })
        .min_by(|a, b| {
            let da = a.1.translation.truncate().distance(world);
            let db = b.1.translation.truncate().distance(world);
            da.total_cmp(&db)
        })
        .map(|(e, ..)| e)
}

/// What is under the pointer: a note, or (while editing) a map object.
fn pick_any(
    world: Vec2,
    editing: bool,
    nodes: &Query<(Entity, &Transform, &GraphNode)>,
    things: &Query<(Entity, &GlobalTransform, &world::MapThing)>,
) -> Option<Entity> {
    pick(world, nodes).or_else(|| editing.then(|| edit::thing_at(world, things)).flatten())
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
    let editing = nav.map_edit;
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
    let editing = nav.map_edit;
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
    mut nav: ResMut<Nav>,
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
    if nav.map_edit {
        // Editing: a click chooses what the inspector edits (or, on empty sky, nothing).
        state.selected = target;
        return;
    }
    let Some(entity) = target else {
        return;
    };
    let Ok(node) = nodes.get(entity) else {
        return;
    };
    nav.open(node.note, None);
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
        if nav.map_edit {
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
    let farthest = state.max_scale();
    zoom_around(&mut transform, &mut projection, anchor, factor, farthest);
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

/// Closest the camera zooms in (its scale: a world unit is two pixels), and the farthest out
/// before the map's size is known.
const MIN_SCALE: f32 = 0.5;
const MAX_SCALE: f32 = 9.0;
/// Closest zoom "fit" goes to.
const FIT_MIN_SCALE: f32 = 0.8;
/// How much farther out than the notes shown the camera may zoom, and the farthest zoom
/// however few they are.
const ZOOM_OUT_ROOM: f32 = 1.4;
const FARTHEST_LEAST: f32 = 1.5;

/// Keeps the camera from zooming out much past the notes shown: works out the farthest zoom
/// from where they sit and the viewport's size, and brings the camera back in when fewer
/// notes show or the viewport grows.
fn limit_zoom(
    mut state: ResMut<GraphState>,
    nodes: Query<&GraphNode>,
    viewport: Query<&ComputedNode, With<GraphViewport>>,
    camera: Query<(&Transform, &Projection), With<GraphCamera>>,
) {
    let Ok(viewport) = viewport.single() else {
        return;
    };
    let size = viewport.size();
    if size.min_element() < 1.0 {
        return;
    }
    let Some(bounds) = nodes
        .iter()
        .filter(|n| !n.hidden)
        .map(|n| Rect::from_center_half_size(n.home, FIT_ROOM))
        .reduce(|a, b| a.union(b))
    else {
        return;
    };
    let farthest =
        ((bounds.size() / size).max_element() * ZOOM_OUT_ROOM).clamp(FARTHEST_LEAST, MAX_SCALE);
    state.farthest = farthest;
    if let Some((_, scale)) = &mut state.camera_goal {
        *scale = scale.min(farthest);
    } else if let Ok((transform, projection)) = camera.single()
        && ortho_scale(projection) > farthest + 0.001
    {
        // Ease back in rather than jump.
        state.camera_goal = Some((transform.translation.truncate(), farthest));
    }
}

/// Zooms by `factor` (< 1 zooms in) keeping the world point `anchor` fixed on screen, no
/// closer than [`MIN_SCALE`] and no farther than `farthest`.
fn zoom_around(
    transform: &mut Transform,
    projection: &mut Projection,
    anchor: Vec2,
    factor: f32,
    farthest: f32,
) {
    let Projection::Orthographic(o) = projection else {
        return;
    };
    let scale = (o.scale * factor).clamp(MIN_SCALE, farthest.max(MIN_SCALE));
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
        nav.view == View::Graph,
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
        let farthest = state.max_scale();
        zoom_around(
            &mut transform,
            &mut projection,
            anchor,
            previous / distance,
            farthest,
        );
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

/// Room kept around each note when framing: the note and its name above.
const FIT_ROOM: Vec2 = Vec2::new(90.0, 60.0);

/// Camera position and zoom that show these notes (at their homes: where they are going).
fn fit<'a>(
    nodes: impl Iterator<Item = (&'a Transform, &'a GraphNode)>,
    viewport: &ComputedNode,
) -> (Vec2, f32) {
    let mut bounds: Option<Rect> = None;
    for (_, node) in nodes.filter(|(_, n)| !n.hidden) {
        let r = Rect::from_center_half_size(node.home, FIT_ROOM);
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
