//! Vertex-coloured triangles, built shape by shape: the lines, the sky and the glows.

use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

/// An empty mesh for vertex-coloured triangles.
pub(crate) fn empty() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
}

#[derive(Default)]
pub(crate) struct Shapes {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Shapes {
    /// A band `half` wide on each side from `a` to `b`.
    pub fn quad(&mut self, a: Vec2, b: Vec2, half: f32, z: f32, color: [f32; 4]) {
        let normal = (b - a).normalize_or_zero().perp() * half;
        let base = self.positions.len() as u32;
        for p in [a + normal, a - normal, b - normal, b + normal] {
            self.positions.push([p.x, p.y, z]);
            self.colors.push(color);
        }
        self.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// A filled regular polygon of `sides` around `center` (a dot, or a disc with many sides).
    pub fn disc(&mut self, center: Vec2, radius: f32, sides: usize, z: f32, color: [f32; 4]) {
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

    /// A circle's outline, `width` wide.
    pub fn circle(&mut self, center: Vec2, radius: f32, width: f32, z: f32, color: [f32; 4]) {
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

    /// Replaces the mesh behind `handle` with these shapes.
    pub fn upload(self, meshes: &mut Assets<Mesh>, handle: &Handle<Mesh>) {
        if let Some(mut mesh) = meshes.get_mut(handle) {
            self.write(&mut mesh);
        }
    }

    fn write(self, mesh: &mut Mesh) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(bevy::mesh::Indices::U32(self.indices));
    }
}
