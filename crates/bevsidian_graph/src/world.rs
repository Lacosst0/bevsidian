//! The sky the stars sit in: a faint celestial grid around the capital (circles and spokes,
//! as on a star chart), a field of background stars that drifts slower than the map when it
//! pans (parallax), soft nebulae where the map puts them, and the regions' names and
//! landmarks. Everything comes from the map's objects ([`MapObject`]) and is rebuilt when the
//! map changes.

use bevy::{
    asset::RenderAssetUsages, camera::visibility::RenderLayers, mesh::PrimitiveTopology,
    prelude::*, text::FontWeight,
};

use bevsidian_core::{map::MapObject, vault::Vault};
use bevsidian_ui::color;

use crate::{GraphNode, GraphState, GraphText, LAYER};

/// A map object in the world: which one (its index among the map's objects) and what it is.
#[derive(Component, Clone, Debug)]
pub(crate) struct MapThing {
    pub index: usize,
    pub object: MapObject,
}

impl MapThing {
    /// How far from its centre a pointer still picks it.
    pub fn reach(&self) -> f32 {
        match &self.object {
            MapObject::Label { text, size, .. } => {
                (text.chars().count() as f32 * size * 0.5).max(40.0)
            }
            MapObject::Landmark { size, .. } => size * 0.6,
            MapObject::Nebula { .. } => HANDLE + 6.0,
        }
    }
}

/// The mesh of the celestial grid.
#[derive(Component)]
pub(crate) struct Ground;

/// The mesh of the background stars (it follows the camera part of the way: parallax).
#[derive(Component)]
pub(crate) struct Starfield;

/// The mesh of the nebulae (see-through).
#[derive(Component)]
pub(crate) struct Nebulae;

/// The handle in the middle of a nebula, shown while editing.
#[derive(Component)]
pub(crate) struct NebulaHandle;

/// Radius of a nebula's handle.
pub(crate) const HANDLE: f32 = 12.0;
/// The celestial grid: a circle every this far from the capital, a spoke every this angle,
/// lines this wide (world units).
const GRID_STEP: f32 = 700.0;
const GRID_SPOKES: usize = 12;
const GRID_WIDTH: f32 = 2.5;
/// Background stars: how many per square of this side, their sizes, and how far the field
/// reaches past the map.
const FIELD_CELL: f32 = 160.0;
const FIELD_PER_CELL: u32 = 3;
const FIELD_SIZES: (f32, f32) = (1.4, 4.0);
/// Smallest a background star is drawn on screen (radius in pixels).
const FIELD_MIN_PX: f32 = 1.0;
const FIELD_MARGIN: f32 = 2400.0;
/// How much of the camera's motion the background follows (0: none, 1: fixed on screen).
const PARALLAX: f32 = 0.35;
/// Upper case, with a space between letters and three between words, the way old maps name
/// their regions.
pub fn spaced(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            word.chars()
                .flat_map(char::to_uppercase)
                .map(String::from)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("   ")
}

/// Rebuilds the map's objects when the vault changes.
pub(crate) fn spawn_world(
    mut commands: Commands,
    vault: Res<Vault>,
    mut state: ResMut<GraphState>,
    old: Query<Entity, With<MapThing>>,
    mut built: Local<Option<u64>>,
) {
    let data = vault.get_untracked();
    if *built == Some(data.revision) {
        return;
    }
    *built = Some(data.revision);
    for entity in &old {
        commands.entity(entity).despawn();
    }
    state.ground_dirty = true;
    let Some(map) = &data.map else {
        return;
    };
    let font = |family: &'static str, size: f32, weight: FontWeight| TextFont {
        font: bevy::text::FontSource::families(family),
        font_size: size.into(),
        weight,
        ..default()
    };
    for (index, object) in map.objects.iter().enumerate() {
        let thing = MapThing {
            index,
            object: object.clone(),
        };
        let at = object.at().extend(Z_OBJECTS);
        let mut entity = commands.spawn((
            thing,
            Transform::from_translation(at),
            Visibility::default(),
            RenderLayers::layer(LAYER),
        ));
        match object {
            MapObject::Label {
                text,
                caption,
                size,
                ..
            } => {
                let (text, caption, size) = (spaced(text), spaced(caption), *size);
                entity.with_children(|label| {
                    label.spawn((
                        GraphText { size },
                        Text2d::new(text),
                        font("Geist, sans-serif", size, FontWeight::SEMIBOLD),
                        TextColor(color::MAP_INK_STRONG),
                        Transform::default(),
                        RenderLayers::layer(LAYER),
                    ));
                    if !caption.is_empty() {
                        let small = (size * 0.5).max(11.0);
                        label.spawn((
                            GraphText { size: small },
                            Text2d::new(caption),
                            font("Geist, sans-serif", small, FontWeight::MEDIUM),
                            TextColor(color::SUBTLE_FOREGROUND),
                            Transform::from_xyz(0.0, -size * 0.95, 0.0),
                            RenderLayers::layer(LAYER),
                        ));
                    }
                });
            }
            MapObject::Landmark { icon, size, .. } => {
                let glyph =
                    bevsidian_core::icons::find(icon).unwrap_or(bevsidian_core::icons::DEFAULT);
                entity.insert((
                    GraphText { size: *size },
                    Text2d::new(glyph.to_string()),
                    font("lucide", *size, FontWeight::NORMAL),
                    bevy::text::LineHeight::RelativeToFont(1.0),
                    TextColor(color::MAP_INK),
                ));
            }
            MapObject::Nebula { .. } => {
                let circle = state.circle.clone();
                let (ring, fill) = (state.paint.ring_hovered.clone(), state.paint.fill.clone());
                entity.insert(Visibility::Hidden).with_children(|handle| {
                    handle.spawn((
                        NebulaHandle,
                        Mesh2d(circle.clone()),
                        MeshMaterial2d(ring),
                        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(HANDLE)),
                        RenderLayers::layer(LAYER),
                    ));
                    handle.spawn((
                        Mesh2d(circle),
                        MeshMaterial2d(fill),
                        Transform::from_xyz(0.0, 0.0, 0.01).with_scale(Vec3::splat(HANDLE - 3.0)),
                        RenderLayers::layer(LAYER),
                    ));
                });
            }
        }
    }
}

/// Depth of labels and landmarks: over the sky, under the lines.
const Z_OBJECTS: f32 = -1.0;

/// Smallest a region's name is drawn on screen (pixels): zoomed out, names grow in the world
/// to stay readable, so the whole map reads like an atlas.
const REGION_NAME_PX: f32 = 15.0;

/// Smallest a landmark is drawn on screen (pixels).
const LANDMARK_PX: f32 = 18.0;

/// Keeps regions' names readable, and landmarks visible, when zoomed out.
pub(crate) fn scale_regions(
    camera: Query<&Projection, With<crate::GraphCamera>>,
    mut things: Query<(&MapThing, &mut Transform)>,
) {
    let Ok(projection) = camera.single() else {
        return;
    };
    let scale = crate::ortho_scale(projection);
    for (thing, mut transform) in &mut things {
        let least = match thing.object {
            MapObject::Label { size, .. } => REGION_NAME_PX / size,
            MapObject::Landmark { size, .. } => LANDMARK_PX / size,
            MapObject::Nebula { .. } => continue,
        };
        let grow = Vec3::splat((scale * least).max(1.0));
        if transform.scale != grow {
            transform.scale = grow;
        }
    }
}

/// Shows nebulae's handles only while editing.
pub(crate) fn show_handles(
    nav: Res<bevsidian_core::nav::Nav>,
    mut things: Query<(&MapThing, &mut Visibility)>,
) {
    let editing = nav.map_edit.get_untracked();
    for (thing, mut visibility) in &mut things {
        if matches!(thing.object, MapObject::Nebula { .. }) {
            visibility.set_if_neq(if editing {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

/// Redraws the sky when the map changes or a nebula is dragged: the grid, the background
/// stars and the nebulae.
pub(crate) fn draw_sky(
    mut state: ResMut<GraphState>,
    things: Query<(&MapThing, &Transform)>,
    nodes: Query<&GraphNode>,
    ground: Query<&Mesh2d, With<Ground>>,
    nebulae: Query<&Mesh2d, With<Nebulae>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !state.ground_dirty {
        return;
    }
    let (Ok(ground), Ok(nebulae)) = (ground.single(), nebulae.single()) else {
        return;
    };
    state.ground_dirty = false;
    let clouds: Vec<(Vec2, f32)> = things
        .iter()
        .filter_map(|(thing, transform)| match thing.object {
            MapObject::Nebula { radius, .. } => Some((transform.translation.truncate(), radius)),
            _ => None,
        })
        .collect();
    let reach = nodes
        .iter()
        .map(|n| n.home.length())
        .chain(clouds.iter().map(|(at, r)| at.length() + r))
        .fold(0.0, f32::max);

    // The grid: circles around the capital, and spokes from the first circle outwards.
    let mut grid = Shapes::default();
    let tint = color::MAP_GRID.to_linear().to_f32_array();
    let circles = (reach / GRID_STEP).ceil() as usize;
    let outer = circles as f32 * GRID_STEP;
    for circle in 1..=circles {
        grid.circle(
            Vec2::ZERO,
            circle as f32 * GRID_STEP,
            GRID_WIDTH,
            -6.0,
            tint,
        );
    }
    for spoke in 0..GRID_SPOKES {
        let direction = Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / GRID_SPOKES as f32);
        grid.capsule(
            direction * GRID_STEP,
            direction * outer,
            GRID_WIDTH / 2.0,
            -6.0,
            tint,
        );
    }
    set_mesh(&mut meshes, &ground.0, grid);

    state.field_reach = outer + FIELD_MARGIN;

    // The nebulae: white in the middle, fading to nothing at the edge.
    let mut clouds_mesh = Shapes::default();
    for &(at, radius) in &clouds {
        clouds_mesh.cloud(at, radius, -5.0);
    }
    set_mesh(&mut meshes, &nebulae.0, clouds_mesh);
}

/// Rebuilds the background stars when the sky changes size or the zoom moves a step: they
/// keep at least [`FIELD_MIN_PX`] on screen, so zoomed out the sky still sparkles.
pub(crate) fn draw_field(
    state: Res<GraphState>,
    camera: Query<&Projection, With<crate::GraphCamera>>,
    field: Query<&Mesh2d, With<Starfield>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<Option<(i32, u32)>>,
) {
    let (Ok(projection), Ok(field)) = (camera.single(), field.single()) else {
        return;
    };
    let scale = crate::ortho_scale(projection);
    let step = (scale.log2() * 2.0).round() as i32;
    let key = (step, state.field_reach as u32);
    if *built == Some(key) || state.field_reach <= 0.0 {
        return;
    }
    *built = Some(key);
    let px = (step as f32 / 2.0).exp2();
    let mut stars = Shapes::default();
    let cells = (state.field_reach / FIELD_CELL).ceil() as i32;
    let (faint, bright) = (
        color::MAP_FIELD_FAINT.to_linear(),
        color::MAP_FIELD_BRIGHT.to_linear(),
    );
    for y in -cells..cells {
        for x in -cells..cells {
            for k in 0..FIELD_PER_CELL {
                let h = hash(x, y, k);
                let at = (Vec2::new(x as f32, y as f32) + Vec2::new(unit(h), unit(h >> 11)))
                    * FIELD_CELL;
                // Most stars are faint; a few are bright (brightness to a high power).
                let brightness = unit(h >> 22).powi(4);
                let tint = faint.mix(&bright, brightness).to_f32_array();
                let size = FIELD_SIZES.0 + (FIELD_SIZES.1 - FIELD_SIZES.0) * brightness;
                stars.disc(
                    at,
                    size.max(FIELD_MIN_PX * px * (0.6 + brightness)),
                    6,
                    -7.0,
                    tint,
                );
            }
        }
    }
    set_mesh(&mut meshes, &field.0, stars);
}

/// The background follows the camera part of the way, so it seems far behind the map.
pub(crate) fn parallax(
    camera: Query<&Transform, With<crate::GraphCamera>>,
    mut field: Query<&mut Transform, (With<Starfield>, Without<crate::GraphCamera>)>,
) {
    let (Ok(camera), Ok(mut field)) = (camera.single(), field.single_mut()) else {
        return;
    };
    let at = (camera.translation.truncate() * PARALLAX).extend(0.0);
    if field.translation != at {
        field.translation = at;
    }
}

fn set_mesh(meshes: &mut Assets<Mesh>, handle: &Handle<Mesh>, shapes: Shapes) {
    if let Some(mut mesh) = meshes.get_mut(handle) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, shapes.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, shapes.colors);
        mesh.insert_indices(bevy::mesh::Indices::U32(shapes.indices));
    }
}

/// A well-mixed hash of a cell and an index.
fn hash(x: i32, y: i32, k: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ k.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

/// The low 11 bits of `h` as a share of 1.
fn unit(h: u32) -> f32 {
    (h & 0x7ff) as f32 / 2048.0
}

/// An empty mesh for vertex-coloured triangles.
pub(crate) fn empty_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
}

#[derive(Default)]
struct Shapes {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Shapes {
    /// A circle's outline, `width` wide.
    fn circle(&mut self, center: Vec2, radius: f32, width: f32, z: f32, color: [f32; 4]) {
        let sides = ((radius / 20.0) as u32).clamp(48, 360);
        let base = self.positions.len() as u32;
        for i in 0..sides {
            let direction = Vec2::from_angle(i as f32 * std::f32::consts::TAU / sides as f32);
            for r in [radius - width / 2.0, radius + width / 2.0] {
                let p = center + direction * r;
                self.positions.push([p.x, p.y, z]);
                self.colors.push(color);
            }
        }
        for i in 0..sides {
            let (a, b) = (base + i * 2, base + (i + 1) % sides * 2);
            self.indices.extend([a, a + 1, b + 1, a, b + 1, b]);
        }
    }

    /// A soft cloud: white in the middle (the material sets how strong), fading out.
    fn cloud(&mut self, center: Vec2, radius: f32, z: f32) {
        const RINGS: u32 = 8;
        const SIDES: u32 = 64;
        let base = self.positions.len() as u32;
        self.positions.push([center.x, center.y, z]);
        self.colors.push([1.0, 1.0, 1.0, 1.0]);
        for ring in 1..=RINGS {
            let r = ring as f32 / RINGS as f32;
            // Smooth all the way to the edge.
            let alpha = (1.0 - r * r).powi(2);
            for side in 0..SIDES {
                let p = center
                    + Vec2::from_angle(side as f32 * std::f32::consts::TAU / SIDES as f32)
                        * radius
                        * r;
                self.positions.push([p.x, p.y, z]);
                self.colors.push([1.0, 1.0, 1.0, alpha]);
            }
        }
        let ring_start = |ring: u32| base + 1 + (ring - 1) * SIDES;
        for side in 0..SIDES {
            let next = (side + 1) % SIDES;
            self.indices
                .extend([base, base + 1 + side, base + 1 + next]);
        }
        for ring in 1..RINGS {
            let (inner, outer) = (ring_start(ring), ring_start(ring + 1));
            for side in 0..SIDES {
                let next = (side + 1) % SIDES;
                self.indices.extend([
                    inner + side,
                    outer + side,
                    outer + next,
                    inner + side,
                    outer + next,
                    inner + next,
                ]);
            }
        }
    }

    /// A band `half` wide on each side from `a` to `b`.
    fn capsule(&mut self, a: Vec2, b: Vec2, half: f32, z: f32, color: [f32; 4]) {
        let normal = (b - a).normalize_or_zero().perp() * half;
        let base = self.positions.len() as u32;
        for p in [a + normal, a - normal, b - normal, b + normal] {
            self.positions.push([p.x, p.y, z]);
            self.colors.push(color);
        }
        self.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn disc(&mut self, center: Vec2, radius: f32, sides: usize, z: f32, color: [f32; 4]) {
        let base = self.positions.len() as u32;
        self.positions.push([center.x, center.y, z]);
        self.colors.push(color);
        let sides = sides as u32;
        for i in 0..sides {
            let p =
                center + Vec2::from_angle(i as f32 * std::f32::consts::TAU / sides as f32) * radius;
            self.positions.push([p.x, p.y, z]);
            self.colors.push(color);
            self.indices
                .extend([base, base + 1 + i, base + 1 + (i + 1) % sides]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_names_are_spaced_out() {
        assert_eq!(spaced("Pixel  plains"), "P I X E L   P L A I N S");
        assert_eq!(spaced(""), "");
    }
}
