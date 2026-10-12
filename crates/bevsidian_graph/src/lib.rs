//! Obsidian-style graph view.
//!
//! Notes, drawn as their icons, are simulated as a force-directed graph in a 2D world on its own render
//! layer. A dedicated camera renders that world into an image that the UI shows in the
//! [`GraphViewport`] node; pointer events on the node are mapped back into the world for
//! hovering, clicking, dragging nodes, panning (with momentum) and zooming (wheel, pinch
//! and buttons). A press only opens a note if the pointer did not move: drags never click.

use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    input::touch::{Touch, Touches},
    picking::events::{
        PointerClick, PointerDrag, PointerDragEnd, PointerMove, PointerOut, PointerPress,
        PointerRelease, PointerScroll,
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
    text::FontWeight,
    ui::UiGlobalTransform,
};

use bevsidian_core::{
    nav::{Nav, View},
    theme::{palette, ui_font},
    vault::{NoteId, Vault},
};

const LAYER: usize = 1;

/// Adds the graph view; place a [`GraphViewport`] node in the UI to show it.
pub fn plugin(app: &mut App) {
    app.init_resource::<GraphState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                attach_viewport,
                toggle_camera,
                resize_target,
                simulate,
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
    pinned: bool,
    neighbors: Vec<Entity>,
}

#[derive(Component)]
struct GraphLabel;

#[derive(Component)]
struct GraphIcon;

/// Text drawn in the graph world, re-rasterized as the camera zooms so it stays sharp.
#[derive(Component)]
struct GraphText {
    /// Size in world units.
    size: f32,
}

#[derive(Resource, Default)]
pub struct GraphState {
    image: Handle<Image>,
    edges: Vec<(Entity, Entity)>,
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
    vault: Res<Vault>,
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
            clear_color: ClearColorConfig::Custom(palette::BG),
            ..default()
        },
        RenderTarget::Image(state.image.clone().into()),
        RenderLayers::layer(LAYER),
        Msaa::Sample4,
    ));

    // A disc of background colour behind each icon, so edges stop at its outline.
    let mask = materials.add(palette::BG);
    let circle = meshes.add(Circle::new(1.0));
    let font = |family: &'static str, size: f32, weight: FontWeight| TextFont {
        font: bevy::text::FontSource::families(family),
        font_size: size.into(),
        weight,
        ..default()
    };

    // Notes start on a circle; the simulation untangles them.
    let note_count = vault.notes.len();
    let mut note_entities = Vec::with_capacity(note_count);
    for (i, note) in vault.notes.iter().enumerate() {
        let angle = i as f32 / note_count.max(1) as f32 * std::f32::consts::TAU;
        let degree = note.outgoing.len() + note.backlinks.len();
        // Better-linked notes are bigger.
        let radius = 14.0 + 4.0 * (degree as f32).sqrt();
        let icon_size = radius * 2.0;
        let id = commands
            .spawn((
                GraphNode {
                    note: i,
                    velocity: Vec2::ZERO,
                    radius,
                    pinned: false,
                    neighbors: Vec::new(),
                },
                Transform::from_translation((Vec2::from_angle(angle) * 180.0).extend(1.0)),
                Visibility::default(),
                RenderLayers::layer(LAYER),
            ))
            .with_children(|node| {
                node.spawn((
                    Mesh2d(circle.clone()),
                    MeshMaterial2d(mask.clone()),
                    Transform::from_xyz(0.0, 0.0, -0.1).with_scale(Vec3::splat(radius * 0.8)),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphIcon,
                    GraphText { size: icon_size },
                    Text2d::new(note.icon.to_string()),
                    font("lucide", icon_size, FontWeight::NORMAL),
                    TextColor(NOTE_COLOR),
                    Transform::default(),
                    RenderLayers::layer(LAYER),
                ));
                node.spawn((
                    GraphLabel,
                    GraphText { size: LABEL_SIZE },
                    Text2d::new(note.name.clone()),
                    font("Inter, sans-serif", LABEL_SIZE, FontWeight::MEDIUM),
                    TextColor(palette::MUTED),
                    Transform::from_xyz(0.0, -radius - 12.0, 0.0),
                    RenderLayers::layer(LAYER),
                ));
            })
            .id();
        note_entities.push(id);
    }
    let mut edges = Vec::new();
    for (i, note) in vault.notes.iter().enumerate() {
        for &target in &note.outgoing {
            edges.push((note_entities[i], note_entities[target]));
        }
    }
    state.edges = edges;
    state.alpha = 1.0;
}

/// Icon colours.
const NOTE_COLOR: Color = Color::srgb(0.62, 0.62, 0.66);
const HOVERED_COLOR: Color = Color::srgb(0.95, 0.95, 1.0);
const DIM_COLOR: Color = Color::srgb(0.3, 0.3, 0.33);

/// Shows the render target in a freshly spawned viewport node and wires pointer input.
fn attach_viewport(
    mut commands: Commands,
    added: Query<Entity, Added<GraphViewport>>,
    mut state: ResMut<GraphState>,
) {
    for entity in &added {
        state.alpha = state.alpha.max(0.6);
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
    let positions: Vec<(Entity, Vec2)> = nodes
        .iter()
        .map(|(e, _, t)| (e, t.translation.truncate()))
        .collect();
    let index = |e: Entity| positions.iter().position(|(p, _)| *p == e);
    let mut forces = vec![Vec2::ZERO; positions.len()];

    // Repulsion between every pair.
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let delta = positions[i].1 - positions[j].1;
            let dist2 = delta.length_squared().max(25.0);
            let f = delta.normalize_or(Vec2::X) * (90000.0 / dist2);
            forces[i] += f;
            forces[j] -= f;
        }
    }
    // Springs along edges.
    for &(a, b) in &state.edges {
        let (Some(i), Some(j)) = (index(a), index(b)) else {
            continue;
        };
        let delta = positions[j].1 - positions[i].1;
        let rest = 160.0;
        let f = delta.normalize_or_zero() * (delta.length() - rest) * 0.6;
        forces[i] += f;
        forces[j] -= f;
    }
    let alpha = state.alpha;
    for (i, (entity, mut node, mut transform)) in nodes.iter_mut().enumerate() {
        debug_assert_eq!(positions[i].0, entity);
        // Gravity towards the centre keeps islands on screen.
        let force = forces[i] - positions[i].1 * 0.08;
        if node.pinned || state.dragging == Some(entity) {
            node.velocity = Vec2::ZERO;
            continue;
        }
        node.velocity = (node.velocity + force * dt * alpha) * 0.85;
        let step = node.velocity * dt * 8.0;
        transform.translation += step.clamp_length_max(40.0).extend(0.0);
    }
    if state.dragging.is_none() {
        state.alpha *= 0.99;
    }
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
) {
    let current = nav.current.get_untracked();
    let focus = state.hovered.or(state.dragging);
    let focus_neighbors = focus.map(|f| neighbors(&state, f)).unwrap_or_default();
    for (entity, mut node, children) in &mut nodes {
        if node.neighbors.is_empty() {
            node.neighbors = neighbors(&state, entity);
        }
        let lit = focus.is_none_or(|f| f == entity || focus_neighbors.contains(&entity));
        let icon = if Some(entity) == focus {
            HOVERED_COLOR
        } else if !lit {
            DIM_COLOR
        } else if node.note == current {
            palette::ACCENT
        } else {
            NOTE_COLOR
        };
        let label = if lit { palette::TEXT } else { palette::FAINT };
        for child in children {
            if let Ok(mut color) = icons.get_mut(*child) {
                color.set_if_neq(TextColor(icon));
            } else if let Ok(mut color) = labels.get_mut(*child) {
                color.set_if_neq(TextColor(label));
            }
        }
    }
}

/// Edge width on screen, in render-target pixels.
const EDGE_WIDTH: f32 = 1.5;

fn draw_edges(
    nav: Res<Nav>,
    state: Res<GraphState>,
    nodes: Query<&Transform, With<GraphNode>>,
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
    let half = EDGE_WIDTH / 2.0 * ortho_scale(projection);
    let focus = state.hovered.or(state.dragging);
    let mut positions = Vec::with_capacity(state.edges.len() * 4);
    let mut colors = Vec::with_capacity(state.edges.len() * 4);
    let mut indices = Vec::with_capacity(state.edges.len() * 6);
    for &(a, b) in &state.edges {
        let (Ok(ta), Ok(tb)) = (nodes.get(a), nodes.get(b)) else {
            continue;
        };
        let highlighted = focus.is_some_and(|f| f == a || f == b);
        let color = if highlighted {
            palette::ACCENT
        } else if focus.is_some() {
            Color::srgb(0.2, 0.2, 0.22)
        } else {
            Color::srgb(0.38, 0.38, 0.42)
        };
        let (pa, pb) = (ta.translation.truncate(), tb.translation.truncate());
        let normal = (pb - pa).normalize_or_zero().perp() * half;
        let base = positions.len() as u32;
        for p in [pa + normal, pa - normal, pb - normal, pb + normal] {
            positions.push([p.x, p.y, 0.0]);
            colors.push(color.to_linear().to_f32_array());
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(bevy::mesh::Indices::U32(indices));
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
        .filter(|(_, t, n)| t.translation.truncate().distance(world) <= n.radius + 12.0)
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
            state.alpha = state.alpha.max(0.3);
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
    // A fling keeps panning (see `camera_motion`); a dragged node just stops.
    if state.dragging.take().is_some() || !state.press.is_some_and(|p| p.moved) {
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
const MAX_SCALE: f32 = 4.0;

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
    nodes: &Query<&Transform, (With<GraphNode>, Without<GraphCamera>)>,
    viewport: Option<&ComputedNode>,
) -> (Vec2, f32) {
    let mut bounds: Option<Rect> = None;
    for transform in nodes {
        let p = transform.translation.truncate();
        // Leave room for the node and its label.
        let r = Rect::from_center_half_size(p, Vec2::new(60.0, 30.0));
        bounds = Some(bounds.map_or(r, |b| b.union(r)));
    }
    let (Some(bounds), Some(viewport)) = (bounds, viewport) else {
        return (Vec2::ZERO, 1.0);
    };
    let size = viewport.size().max(Vec2::ONE);
    let scale = (bounds.size() / size).max_element() * 1.1;
    (bounds.center(), scale.clamp(MIN_SCALE, MAX_SCALE))
}

/// Zoom buttons over the graph (the only way to zoom on touch screens).
pub fn zoom_controls(_nav: Nav) -> impl Scene {
    fn button(label: &'static str, factor: f32) -> impl Scene {
        let text = Text::new(label);
        bsn! {
            Node {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
            }
            BackgroundColor(palette::BG_SIDEBAR)
            BorderColor::all(Color::srgb(0.25, 0.25, 0.27))
            bevy::ui_widgets::Button
            bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
            on(move |_: On<bevy::ui_widgets::Activate>,
                     camera: Query<(&Transform, &Projection), With<GraphCamera>>,
                     nodes: Query<&Transform, (With<GraphNode>, Without<GraphCamera>)>,
                     viewport: Query<&ComputedNode, With<GraphViewport>>,
                     mut state: ResMut<GraphState>| {
                let Ok((transform, projection)) = camera.single() else {
                    return;
                };
                let (position, scale) = state
                    .camera_goal
                    .unwrap_or((transform.translation.truncate(), ortho_scale(projection)));
                state.pan_velocity = Vec2::ZERO;
                state.camera_goal = Some(if factor == 0.0 {
                    fit_all(&nodes, viewport.single().ok())
                } else {
                    (position, (scale * factor).clamp(MIN_SCALE, MAX_SCALE))
                });
            })
            Children [
                text
                @{ui_font(if label.len() > 1 { 12.0 } else { 18.0 }, FontWeight::MEDIUM)}
                TextColor(palette::TEXT)
                Pickable::IGNORE
            ]
        }
    }
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(14.0),
            bottom: Val::Px(14.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            @{button("+", 0.8)}
            --
            @{button("−", 1.25)}
            --
            @{button("Fit", 0.0)}
        ]
    }
}
