//! Obsidian-style graph view.
//!
//! Notes (and tags) are simulated as a force-directed graph in a 2D world on its own render
//! layer. A dedicated camera renders that world into an image that the UI shows in the
//! [`GraphViewport`] node; pointer events on the node are mapped back into the world for
//! hovering, clicking, dragging nodes, panning and zooming.

use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    picking::events::{
        PointerClick, PointerDrag, PointerDragEnd, PointerDragStart, PointerMove, PointerOut,
        PointerScroll,
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
    text::FontWeight,
    ui::UiGlobalTransform,
};

use crate::{
    nav::{Nav, View},
    ui::palette,
    vault::Vault,
};

const LAYER: usize = 1;

pub fn plugin(app: &mut App) {
    app.init_gizmo_group::<GraphGizmos>()
        .init_resource::<GraphState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                attach_viewport,
                toggle_camera,
                resize_target,
                simulate,
                style_nodes,
                draw_edges,
            )
                .chain(),
        );
}

/// Gizmos drawn only by the graph camera.
#[derive(Default, Reflect, GizmoConfigGroup)]
struct GraphGizmos;

/// UI node that displays the graph.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct GraphViewport;

#[derive(Component)]
struct GraphCamera;

/// What a graph node represents.
#[derive(Clone, Debug, PartialEq, Eq)]
enum NodeKind {
    Note(usize),
    Tag(String),
}

#[derive(Component)]
struct GraphNode {
    kind: NodeKind,
    velocity: Vec2,
    radius: f32,
    pinned: bool,
    neighbors: Vec<Entity>,
}

#[derive(Component)]
struct GraphLabel;

#[derive(Resource, Default)]
pub(crate) struct GraphState {
    image: Handle<Image>,
    edges: Vec<(Entity, Entity, bool)>,
    hovered: Option<Entity>,
    dragging: Option<Entity>,
    /// Simulation "temperature": decays to zero, re-heated by interaction.
    alpha: f32,
    materials: Option<GraphMaterials>,
}

#[derive(Clone)]
struct GraphMaterials {
    note: Handle<ColorMaterial>,
    tag: Handle<ColorMaterial>,
    current: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    dim: Handle<ColorMaterial>,
}

impl GraphState {
    /// Whether the simulation or a drag still needs frames.
    pub(crate) fn animating(&self) -> bool {
        self.alpha >= 0.002 || self.dragging.is_some()
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut gizmos: ResMut<GizmoConfigStore>,
    mut state: ResMut<GraphState>,
    vault: Res<Vault>,
) {
    let (config, _) = gizmos.config_mut::<GraphGizmos>();
    config.render_layers = RenderLayers::layer(LAYER);
    config.line.width = 1.5;

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

    let mats = GraphMaterials {
        note: materials.add(Color::srgb(0.62, 0.62, 0.66)),
        tag: materials.add(palette::ACCENT.with_alpha(0.55)),
        current: materials.add(palette::ACCENT),
        hovered: materials.add(Color::srgb(0.95, 0.95, 1.0)),
        dim: materials.add(Color::srgb(0.3, 0.3, 0.33)),
    };
    let circle = meshes.add(Circle::new(1.0));
    let font = TextFont {
        font: bevy::text::FontSource::families("Inter, sans-serif"),
        font_size: 13.0.into(),
        weight: FontWeight::MEDIUM,
        ..default()
    };

    // Notes on a circle, tags on an inner circle; the simulation untangles them.
    let note_count = vault.notes.len();
    let mut note_entities = Vec::with_capacity(note_count);
    let spawn_node = |commands: &mut Commands,
                      kind: NodeKind,
                      label: String,
                      pos: Vec2,
                      radius: f32,
                      material: Handle<ColorMaterial>| {
        commands
            .spawn((
                GraphNode {
                    kind,
                    velocity: Vec2::ZERO,
                    radius,
                    pinned: false,
                    neighbors: Vec::new(),
                },
                Mesh2d(circle.clone()),
                MeshMaterial2d(material),
                Transform::from_translation(pos.extend(1.0)).with_scale(Vec3::splat(radius)),
                RenderLayers::layer(LAYER),
            ))
            .with_child((
                GraphLabel,
                Text2d::new(label),
                font.clone(),
                TextColor(palette::MUTED),
                // Counter the parent's radius scale so labels keep their size.
                Transform::from_xyz(0.0, -1.0 - 14.0 / radius, 0.0)
                    .with_scale(Vec3::splat(1.0 / radius)),
                RenderLayers::layer(LAYER),
            ))
            .id()
    };
    for (i, note) in vault.notes.iter().enumerate() {
        let angle = i as f32 / note_count.max(1) as f32 * std::f32::consts::TAU;
        let degree = note.outgoing.len() + note.backlinks.len();
        let radius = 7.0 + 2.0 * (degree as f32).sqrt() * 2.0;
        let id = spawn_node(
            &mut commands,
            NodeKind::Note(i),
            note.name.clone(),
            Vec2::from_angle(angle) * 180.0,
            radius,
            mats.note.clone(),
        );
        note_entities.push(id);
    }
    let mut edges = Vec::new();
    for (i, note) in vault.notes.iter().enumerate() {
        for &target in &note.outgoing {
            edges.push((note_entities[i], note_entities[target], false));
        }
    }
    for (j, (tag, notes)) in vault.tags.iter().enumerate() {
        let angle = j as f32 / vault.tags.len().max(1) as f32 * std::f32::consts::TAU + 0.3;
        let id = spawn_node(
            &mut commands,
            NodeKind::Tag(tag.clone()),
            format!("#{tag}"),
            Vec2::from_angle(angle) * 90.0,
            5.0,
            mats.tag.clone(),
        );
        for &note in notes {
            edges.push((id, note_entities[note], true));
        }
    }
    state.edges = edges;
    state.alpha = 1.0;
    state.materials = Some(mats);
}

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
            .observe(on_click)
            .observe(on_drag_start)
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
    for &(a, b, tag) in &state.edges {
        let (Some(i), Some(j)) = (index(a), index(b)) else {
            continue;
        };
        let delta = positions[j].1 - positions[i].1;
        let rest = if tag { 90.0 } else { 160.0 };
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

fn neighbors(state: &GraphState, entity: Entity) -> Vec<Entity> {
    state
        .edges
        .iter()
        .filter_map(|&(a, b, _)| {
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
    mut nodes: Query<(
        Entity,
        &mut GraphNode,
        &mut MeshMaterial2d<ColorMaterial>,
        &Children,
    )>,
    mut labels: Query<&mut TextColor, With<GraphLabel>>,
) {
    let Some(mats) = &state.materials else {
        return;
    };
    let current = nav.current.get_untracked();
    let focus = state.hovered.or(state.dragging);
    let focus_neighbors = focus.map(|f| neighbors(&state, f)).unwrap_or_default();
    for (entity, mut node, mut material, children) in &mut nodes {
        if node.neighbors.is_empty() {
            node.neighbors = neighbors(&state, entity);
        }
        let lit = focus.is_none_or(|f| f == entity || focus_neighbors.contains(&entity));
        let wanted = if Some(entity) == focus {
            &mats.hovered
        } else if !lit {
            &mats.dim
        } else if node.kind == NodeKind::Note(current) {
            &mats.current
        } else if matches!(node.kind, NodeKind::Tag(_)) {
            &mats.tag
        } else {
            &mats.note
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
        for child in children {
            if let Ok(mut color) = labels.get_mut(*child) {
                let c = if lit { palette::TEXT } else { palette::FAINT };
                if color.0 != c {
                    color.0 = c;
                }
            }
        }
    }
}

fn draw_edges(
    nav: Res<Nav>,
    state: Res<GraphState>,
    nodes: Query<&Transform, With<GraphNode>>,
    mut gizmos: Gizmos<GraphGizmos>,
) {
    if nav.view.get_untracked() != View::Graph {
        return;
    }
    let focus = state.hovered.or(state.dragging);
    for &(a, b, tag) in &state.edges {
        let (Ok(ta), Ok(tb)) = (nodes.get(a), nodes.get(b)) else {
            continue;
        };
        let highlighted = focus.is_some_and(|f| f == a || f == b);
        let color = if highlighted {
            palette::ACCENT
        } else if focus.is_some() {
            Color::srgb(0.2, 0.2, 0.22)
        } else if tag {
            palette::ACCENT.with_alpha(0.25)
        } else {
            Color::srgb(0.38, 0.38, 0.42)
        };
        gizmos.line_2d(ta.translation.truncate(), tb.translation.truncate(), color);
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

fn on_click(
    event: On<PointerClick>,
    state: Res<GraphState>,
    nodes: Query<&GraphNode>,
    nav: Res<Nav>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Some(entity) = state.hovered else {
        return;
    };
    match nodes.get(entity).map(|n| &n.kind) {
        Ok(NodeKind::Note(id)) => nav.open(*id, None),
        Ok(NodeKind::Tag(tag)) => nav.tag.set(Some(tag.clone())),
        Err(_) => {}
    }
}

fn on_drag_start(event: On<PointerDragStart>, mut state: ResMut<GraphState>) {
    if event.button == PointerButton::Primary {
        state.dragging = state.hovered;
        state.alpha = state.alpha.max(0.3);
    }
}

fn on_drag(
    event: On<PointerDrag>,
    nodes_ui: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut camera: Query<(&Camera, &mut Transform, &Projection), With<GraphCamera>>,
    mut nodes: Query<&mut Transform, (With<GraphNode>, Without<GraphCamera>)>,
    state: Res<GraphState>,
) {
    let Ok((node, _)) = nodes_ui.get(event.entity) else {
        return;
    };
    let Ok((_, mut cam, projection)) = camera.single_mut() else {
        return;
    };
    let scale = match projection {
        Projection::Orthographic(o) => o.scale,
        _ => 1.0,
    };
    // Logical px → target px → world units.
    let delta = event.delta / node.inverse_scale_factor * scale;
    let delta = Vec2::new(delta.x, -delta.y);
    match state.dragging {
        Some(entity) => {
            if let Ok(mut transform) = nodes.get_mut(entity) {
                transform.translation += delta.extend(0.0);
            }
        }
        None => cam.translation -= delta.extend(0.0),
    }
}

fn on_drag_end(_: On<PointerDragEnd>, mut state: ResMut<GraphState>) {
    state.dragging = None;
}

fn on_scroll(event: On<PointerScroll>, mut camera: Query<&mut Projection, With<GraphCamera>>) {
    let Ok(mut projection) = camera.single_mut() else {
        return;
    };
    if let Projection::Orthographic(o) = &mut *projection {
        let factor = (1.0 - event.y.signum() * 0.1).clamp(0.5, 1.5);
        o.scale = (o.scale * factor).clamp(0.25, 4.0);
    }
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
                     mut camera: Query<(&mut Projection, &mut Transform), With<GraphCamera>>| {
                if let Ok((mut projection, mut transform)) = camera.single_mut()
                    && let Projection::Orthographic(o) = &mut *projection
                {
                    if factor == 0.0 {
                        o.scale = 1.0;
                        transform.translation = Vec3::ZERO;
                    } else {
                        o.scale = (o.scale * factor).clamp(0.25, 4.0);
                    }
                }
            })
            Children [
                text
                @{crate::ui::ui_font(if label.len() > 1 { 12.0 } else { 18.0 }, FontWeight::MEDIUM)}
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
