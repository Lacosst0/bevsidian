//! What a note is drawn with: its [`Shape`], filled, with an outline around it (the note's
//! icon sits inside). Meshes are unit sized and scaled to each note.

use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

use bevsidian_core::map::Shape;

/// The corners of a shape of radius 1, as drawn: polygons are a little larger than the
/// circle so they look the same size.
pub fn outline(shape: Shape) -> Vec<Vec2> {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI, TAU};
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
        Shape::Star => (0..10)
            .map(|i| {
                let size = if i % 2 == 0 { 1.3 } else { 0.62 };
                Vec2::from_angle(FRAC_PI_2 + i as f32 * PI / 5.0) * size
            })
            .collect(),
    }
}

/// The hairline outline of a shape of radius 1, `width` thick (inside the radius).
pub fn ring(shape: Shape, width: f32) -> Mesh {
    let outer = outline(shape);
    let scale = 1.0 - width;
    let count = outer.len() as u32;
    let mut positions: Vec<[f32; 3]> = outer.iter().map(|c| [c.x, c.y, 0.0]).collect();
    positions.extend(outer.iter().map(|c| [c.x * scale, c.y * scale, 0.0]));
    let indices: Vec<u32> = (0..count)
        .flat_map(|i| {
            let next = (i + 1) % count;
            [i, next, count + i, next, count + next, count + i]
        })
        .collect();
    flat(positions, indices)
}

/// A filled shape of radius 1: a fan of triangles from its centre (every shape here is
/// star-shaped around it, so the fan covers it exactly).
pub fn fill(shape: Shape) -> Mesh {
    let corners = outline(shape);
    let mut positions = vec![[0.0, 0.0, 0.0]];
    positions.extend(corners.iter().map(|c| [c.x, c.y, 0.0]));
    let count = corners.len() as u32;
    let indices = (0..count)
        .flat_map(|i| [0, 1 + i, 1 + (i + 1) % count])
        .collect();
    flat(positions, indices)
}

/// How far a shape reaches above its centre, as a share of its radius.
pub fn top(shape: Shape) -> f32 {
    outline(shape).iter().map(|p| p.y).fold(0.0, f32::max)
}

fn flat(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5, 0.5]; count])
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_outline_is_a_closed_band() {
        for shape in Shape::ALL {
            let corners = outline(shape).len();
            let ring = ring(shape, 0.1);
            assert_eq!(ring.count_vertices(), corners * 2, "{shape:?}");
            assert_eq!(ring.indices().unwrap().len(), corners * 6, "{shape:?}");
        }
    }

    #[test]
    fn every_fill_is_a_closed_fan() {
        for shape in Shape::ALL {
            let corners = outline(shape).len();
            let fill = fill(shape);
            assert_eq!(fill.count_vertices(), corners + 1, "{shape:?}");
            assert_eq!(fill.indices().unwrap().len(), corners * 3, "{shape:?}");
            assert!((0.8..1.4).contains(&top(shape)), "{shape:?}");
        }
    }
}
