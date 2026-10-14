//! The sky the stars sit in: a faint celestial grid around the capital (circles and spokes,
//! as on a star chart), a field of background stars that drifts slower than the map when it
//! pans (parallax), soft nebulae where the map puts them, and its labels and
//! landmarks. Everything comes from the map's objects ([`MapObject`]) and is rebuilt when the
//! map changes.

use bevy::{camera::visibility::RenderLayers, prelude::*, text::FontWeight};

use bevsidian_core::{map::MapObject, vault::Vault};
use bevsidian_ui::color;

use crate::{GraphNode, GraphState, GraphText, LAYER, mesh::Shapes};

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
pub(crate) struct Grid;

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
/// Upper case, with a space between letters and three between words, as star charts letter
/// their constellations.
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
    state.sky_dirty = true;
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
                // On a child: the text's scale keeps it sharp ([`GraphText`]), the
                // landmark's keeps it a size on screen.
                let size = *size;
                entity.with_children(|landmark| {
                    landmark.spawn((
                        GraphText { size },
                        Text2d::new(glyph.to_string()),
                        font("lucide", size, FontWeight::NORMAL),
                        bevy::text::LineHeight::RelativeToFont(1.0),
                        TextColor(color::MAP_INK),
                        Transform::default(),
                        RenderLayers::layer(LAYER),
                    ));
                });
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

/// Depths, back to front: the background stars, the grid, the nebulae, then labels and
/// landmarks (all under the lines and the notes).
const Z_FIELD: f32 = -7.0;
const Z_GRID: f32 = -6.0;
const Z_NEBULAE: f32 = -5.0;
const Z_OBJECTS: f32 = -1.0;

/// Smallest a label is drawn on screen (pixels): zoomed out, labels grow in the world to
/// stay readable.
const LABEL_PX: f32 = 15.0;

/// Smallest a landmark is drawn on screen (pixels).
const LANDMARK_PX: f32 = 18.0;

/// Keeps labels readable, and landmarks visible, when zoomed out.
pub(crate) fn scale_objects(
    camera: Query<&Projection, With<crate::GraphCamera>>,
    mut things: Query<(&MapThing, &mut Transform)>,
) {
    let Ok(projection) = camera.single() else {
        return;
    };
    let scale = crate::ortho_scale(projection);
    for (thing, mut transform) in &mut things {
        let least = match thing.object {
            MapObject::Label { size, .. } => LABEL_PX / size,
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
    grid_mesh: Query<&Mesh2d, With<Grid>>,
    nebulae: Query<&Mesh2d, With<Nebulae>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !state.sky_dirty {
        return;
    }
    let (Ok(grid_mesh), Ok(nebulae)) = (grid_mesh.single(), nebulae.single()) else {
        return;
    };
    state.sky_dirty = false;
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
            Z_GRID,
            tint,
        );
    }
    for spoke in 0..GRID_SPOKES {
        let direction = Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / GRID_SPOKES as f32);
        grid.quad(
            direction * GRID_STEP,
            direction * outer,
            GRID_WIDTH / 2.0,
            Z_GRID,
            tint,
        );
    }
    grid.upload(&mut meshes, &grid_mesh.0);

    state.field_reach = outer + FIELD_MARGIN;

    // The nebulae: white in the middle, fading to nothing at the edge.
    let mut clouds_mesh = Shapes::default();
    for &(at, radius) in &clouds {
        clouds_mesh.fade(at, radius, (8, 64), Z_NEBULAE, |r| (1.0 - r * r).powi(2));
    }
    clouds_mesh.upload(&mut meshes, &nebulae.0);
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
                    Z_FIELD,
                    tint,
                );
            }
        }
    }
    stars.upload(&mut meshes, &field.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_spaced_out() {
        assert_eq!(spaced("Pixel  plains"), "P I X E L   P L A I N S");
        assert_eq!(spaced(""), "");
    }
}
