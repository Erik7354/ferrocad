use crate::{
    length::Length,
    mesh::{Mesh, Point3},
    solid::Solid,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cuboid {
    pub width: Length,
    pub depth: Length,
    pub height: Length,
}

impl Cuboid {
    pub const fn new(width: Length, depth: Length, height: Length) -> Self {
        Self {
            width,
            depth,
            height,
        }
    }

    pub const fn cube(size: Length) -> Self {
        Self::new(size, size, size)
    }
}

impl Solid for Cuboid {
    fn mesh(&self, _tolerance: Length) -> Mesh {
        let x = self.width.as_mm_f64() / 2.0;
        let y = self.depth.as_mm_f64() / 2.0;
        let z = self.height.as_mm_f64() / 2.0;

        let vertices = vec![
            // Bottom
            Point3::new(-x, -y, -z), // 0
            Point3::new(x, -y, -z),  // 1
            Point3::new(x, y, -z),   // 2
            Point3::new(-x, y, -z),  // 3
            // Top
            Point3::new(-x, -y, z), // 4
            Point3::new(x, -y, z),  // 5
            Point3::new(x, y, z),   // 6
            Point3::new(-x, y, z),  // 7
        ];

        let triangles = vec![
            // Bottom (-Z)
            [0, 2, 1],
            [0, 3, 2],
            // Top (+Z)
            [4, 5, 6],
            [4, 6, 7],
            // Front (-Y)
            [0, 1, 5],
            [0, 5, 4],
            // Back (+Y)
            [3, 7, 6],
            [3, 6, 2],
            // Left (-X)
            [0, 4, 7],
            [0, 7, 3],
            // Right (+X)
            [1, 2, 6],
            [1, 6, 5],
        ];

        Mesh::new(vertices, triangles)
    }
}
