//! What the map shows besides notes and roads: the land (overlapping discs merged into one
//! coastline, with shallows around it, on a dotted sea), the regions' names and landmarks.
//! Everything comes from the map's objects ([`MapObject`]) and is rebuilt when the map
//! changes.

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
            MapObject::Land { .. } => LAND_HANDLE + 6.0,
        }
    }
}

/// The mesh of the sea, the shallows, the coast and the land.
#[derive(Component)]
pub(crate) struct Ground;

/// The handle in the middle of a piece of land, shown while editing.
#[derive(Component)]
pub(crate) struct LandHandle;

/// Radius of a land handle.
pub(crate) const LAND_HANDLE: f32 = 12.0;
/// How far the shallows and the coast reach past the land.
const SHALLOWS: f32 = 34.0;
const COAST: f32 = 4.0;
/// Spacing and size of the dots of the sea.
const SEA_GAP: f32 = 64.0;
const SEA_DOT: f32 = 2.2;
/// Discs closer than this (beyond touching) are joined, by a neck this share of the smaller
/// one's radius wide (on each side).
const NECK_REACH: f32 = 120.0;
const NECK: f32 = 0.62;
/// How far the sea reaches past the land and the notes.
const SEA_MARGIN: f32 = 900.0;

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
            MapObject::Land { .. } => {
                let circle = state.circle.clone();
                let (ring, fill) = (state.paint.ring_hovered.clone(), state.paint.fill.clone());
                entity.insert(Visibility::Hidden).with_children(|handle| {
                    handle.spawn((
                        LandHandle,
                        Mesh2d(circle.clone()),
                        MeshMaterial2d(ring),
                        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(LAND_HANDLE)),
                        RenderLayers::layer(LAYER),
                    ));
                    handle.spawn((
                        Mesh2d(circle),
                        MeshMaterial2d(fill),
                        Transform::from_xyz(0.0, 0.0, 0.01)
                            .with_scale(Vec3::splat(LAND_HANDLE - 3.0)),
                        RenderLayers::layer(LAYER),
                    ));
                });
            }
        }
    }
}

/// Depth of labels and landmarks: over the land, under the roads.
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
            MapObject::Land { .. } => continue,
        };
        let grow = Vec3::splat((scale * least).max(1.0));
        if transform.scale != grow {
            transform.scale = grow;
        }
    }
}

/// Shows land handles only while editing.
pub(crate) fn show_handles(
    nav: Res<bevsidian_core::nav::Nav>,
    mut things: Query<(&MapThing, &mut Visibility)>,
) {
    let editing = nav.map_edit.get_untracked();
    for (thing, mut visibility) in &mut things {
        if matches!(thing.object, MapObject::Land { .. }) {
            visibility.set_if_neq(if editing {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

/// Redraws the ground when the land moves (the map changed, or a piece is being dragged).
pub(crate) fn draw_ground(
    mut state: ResMut<GraphState>,
    things: Query<(&MapThing, &Transform)>,
    nodes: Query<&GraphNode>,
    ground: Query<&Mesh2d, With<Ground>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !state.ground_dirty {
        return;
    }
    let Ok(mesh) = ground.single() else {
        return;
    };
    state.ground_dirty = false;
    let lands: Vec<(Vec2, f32)> = things
        .iter()
        .filter_map(|(thing, transform)| match thing.object {
            MapObject::Land { radius, .. } => Some((transform.translation.truncate(), radius)),
            _ => None,
        })
        .collect();
    let mut bounds: Option<Rect> = None;
    let mut grow = |r: Rect| bounds = Some(bounds.map_or(r, |b| b.union(r)));
    for &(at, radius) in &lands {
        grow(Rect::from_center_half_size(at, Vec2::splat(radius)));
    }
    for node in &nodes {
        grow(Rect::from_center_half_size(node.home, Vec2::splat(1.0)));
    }
    let mut out = Shapes::default();
    if let Some(bounds) = bounds {
        let sea = bounds.inflate(SEA_MARGIN);
        let dot = color::CARD.to_linear().to_f32_array();
        let (cols, rows) = (
            (sea.width() / SEA_GAP) as usize,
            (sea.height() / SEA_GAP) as usize,
        );
        // On a fixed grid, so the dots stay put when the land changes.
        let origin = (sea.min / SEA_GAP).floor() * SEA_GAP;
        for row in 0..=rows {
            for col in 0..=cols {
                let at = origin + Vec2::new(col as f32, row as f32) * SEA_GAP;
                out.disc(at, SEA_DOT, 6, -6.0, dot);
            }
        }
    }
    for (grow, z, tint) in [
        (SHALLOWS, -5.0, color::MAP_SHALLOWS),
        (COAST, -4.9, color::MAP_COAST),
        (0.0, -4.8, color::MAP_LAND),
    ] {
        let rgba = tint.to_linear().to_f32_array();
        for (i, &(at, radius)) in lands.iter().enumerate() {
            let r = radius + grow;
            out.disc(at, r, ((r / 12.0) as usize).clamp(24, 160), z, rgba);
            // Discs that nearly touch are joined by a neck of land, so no narrow inlet
            // cuts between them.
            for &(other, other_radius) in &lands[i + 1..] {
                let gap = at.distance(other) - radius - other_radius;
                if gap < NECK_REACH {
                    let width = radius.min(other_radius) * NECK + grow;
                    out.capsule(at, other, width, z, rgba);
                }
            }
        }
    }
    if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, out.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, out.colors);
        mesh.insert_indices(bevy::mesh::Indices::U32(out.indices));
    }
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
