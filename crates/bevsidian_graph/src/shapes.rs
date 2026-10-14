//! The outlines notes are drawn in: one unit-sized mesh per [`Shape`], scaled to a note's
//! radius (once for its outline, once, a little smaller, for its fill).

use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

use bevsidian_core::map::Shape;

/// The corners of a shape of radius 1, as drawn: polygons are a little larger than the
/// circle so they look the same size.
pub fn outline(shape: Shape) -> Vec<Vec2> {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8, PI, TAU};
    let regular = |sides: usize, size: f32, turn: f32| -> Vec<Vec2> {
        (0..sides)
            .map(|i| Vec2::from_angle(turn + i as f32 * TAU / sides as f32) * size)
            .collect()
    };
    match shape {
        Shape::Circle => regular(48, 1.0, 0.0),
        Shape::Diamond => regular(4, 1.2, FRAC_PI_2),
        Shape::Square => regular(4, 1.2, FRAC_PI_4),
        Shape::Hexagon => regular(6, 1.1, FRAC_PI_6),
        Shape::Pentagon => regular(5, 1.12, FRAC_PI_2),
        Shape::Octagon => regular(8, 1.06, FRAC_PI_8),
        Shape::Star => (0..10)
            .map(|i| {
                let size = if i % 2 == 0 { 1.3 } else { 0.62 };
                Vec2::from_angle(FRAC_PI_2 + i as f32 * PI / 5.0) * size
            })
            .collect(),
    }
}

/// A filled shape: a fan of triangles from its centre (every shape here is star-shaped around
/// it, so the fan covers it exactly).
pub fn mesh(shape: Shape) -> Mesh {
    let corners = outline(shape);
    let mut positions = vec![[0.0, 0.0, 0.0]];
    positions.extend(corners.iter().map(|c| [c.x, c.y, 0.0]));
    let count = corners.len() as u32;
    let indices: Vec<u32> = (0..count)
        .flat_map(|i| [0, 1 + i, 1 + (i + 1) % count])
        .collect();
    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs = vec![[0.5, 0.5]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

/// How far the fill sits inside the outline, for an outline `width` wide: the inset that
/// gives the same width along each side of a polygon (its sides are closer to the centre than
/// its corners).
pub fn inset(shape: Shape, width: f32) -> f32 {
    let sides = match shape {
        Shape::Circle => return width,
        Shape::Diamond | Shape::Square => 4.0,
        Shape::Hexagon => 6.0,
        Shape::Pentagon => 5.0,
        Shape::Octagon => 8.0,
        // The star's inner corners make its sides steep.
        Shape::Star => 2.4,
    };
    width / (std::f32::consts::PI / sides).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_is_a_closed_fan() {
        for shape in Shape::ALL {
            let mesh = mesh(shape);
            let corners = outline(shape).len();
            assert_eq!(mesh.count_vertices(), corners + 1, "{shape:?}");
            assert_eq!(mesh.indices().unwrap().len(), corners * 3, "{shape:?}");
            assert!(inset(shape, 2.0) >= 2.0);
        }
    }
}
