//! What a note is drawn with: a star (a bright core in a soft glow, with diffraction spikes
//! for the brightest), and the hairline outline of its [`Shape`] around it. Meshes are unit
//! sized and scaled to each note.

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
    flat(positions, indices, None)
}

/// A soft glow of radius 1: opaque white in the middle, fading to nothing at the edge (the
/// material's colour tints it and sets how strong it is).
pub fn glow() -> Mesh {
    const RINGS: usize = 6;
    const SIDES: usize = 40;
    let mut positions = vec![[0.0, 0.0, 0.0]];
    let mut colors = vec![[1.0, 1.0, 1.0, 1.0]];
    for ring in 1..=RINGS {
        let r = ring as f32 / RINGS as f32;
        // Fades like a star's light: fast near the core, a long faint tail.
        let alpha = (1.0 - r).powi(3);
        for side in 0..SIDES {
            let p = Vec2::from_angle(side as f32 * std::f32::consts::TAU / SIDES as f32) * r;
            positions.push([p.x, p.y, 0.0]);
            colors.push([1.0, 1.0, 1.0, alpha]);
        }
    }
    let ring_start = |ring: usize| (1 + (ring - 1) * SIDES) as u32;
    let mut indices = Vec::new();
    for side in 0..SIDES as u32 {
        let next = (side + 1) % SIDES as u32;
        indices.extend([0, 1 + side, 1 + next]);
    }
    for ring in 1..RINGS {
        let (inner, outer) = (ring_start(ring), ring_start(ring + 1));
        for side in 0..SIDES as u32 {
            let next = (side + 1) % SIDES as u32;
            indices.extend([
                inner + side,
                outer + side,
                outer + next,
                inner + side,
                outer + next,
                inner + next,
            ]);
        }
    }
    flat(positions, indices, Some(colors))
}

/// Diffraction spikes of length 1: four thin points, bright at the centre, fading to their
/// tips (two long ones across, two shorter up and down… as a telescope draws a bright star).
pub fn spikes() -> Mesh {
    let mut positions = vec![[0.0, 0.0, 0.0]];
    let mut colors = vec![[1.0, 1.0, 1.0, 1.0]];
    let mut indices = Vec::new();
    for (i, length) in [1.0_f32, 0.7, 1.0, 0.7].into_iter().enumerate() {
        let direction = Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_2);
        let side = direction.perp() * 0.035;
        let base = positions.len() as u32;
        for (p, alpha) in [(side, 0.8), (direction * length, 0.0), (-side, 0.8)] {
            positions.push([p.x, p.y, 0.0]);
            colors.push([1.0, 1.0, 1.0, alpha]);
        }
        indices.extend([0, base, base + 1, 0, base + 1, base + 2]);
    }
    flat(positions, indices, Some(colors))
}

fn flat(positions: Vec<[f32; 3]>, indices: Vec<u32>, colors: Option<Vec<[f32; 4]>>) -> Mesh {
    let count = positions.len();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5, 0.5]; count])
    .with_inserted_indices(bevy::mesh::Indices::U32(indices));
    if let Some(colors) = colors {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    mesh
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
    fn glows_fade_out_and_spikes_have_four_points() {
        let glow = glow();
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(colors)) =
            glow.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("a glow has vertex colours");
        };
        assert!(
            (colors[0][3] - 1.0).abs() < f32::EPSILON,
            "bright in the middle"
        );
        assert!(
            colors.last().unwrap()[3].abs() < f32::EPSILON,
            "gone at the edge"
        );
        assert_eq!(spikes().indices().unwrap().len(), 4 * 6);
    }
}
