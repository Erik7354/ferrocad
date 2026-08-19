use crate::{
    length::Length,
    mesh::{Mesh, Point3},
    solid::Solid,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cylinder {
    pub radius: Length,
    pub height: Length,
}

impl Cylinder {
    pub const fn new(radius: Length, height: Length) -> Self {
        Self { radius, height }
    }
}

impl Solid for Cylinder {
    fn mesh(&self, tolerance: Length) -> Mesh {
        let radius = self.radius.as_mm_f64();
        let half_height = self.height.as_mm_f64() / 2.0;
        let segments = super::circle_segments(radius, tolerance.as_mm_f64());

        let mut vertices = Vec::with_capacity(2 * segments + 2);
        vertices.push(Point3::new(0.0, 0.0, -half_height));
        for i in 0..segments {
            vertices.push(rim_point(radius, -half_height, i, segments));
        }
        vertices.push(Point3::new(0.0, 0.0, half_height));
        for i in 0..segments {
            vertices.push(rim_point(radius, half_height, i, segments));
        }

        let bottom_rim = 1;
        let top_center = segments + 1;
        let top_rim = segments + 2;

        let mut triangles = Vec::with_capacity(4 * segments);
        for i in 0..segments {
            let next = (i + 1) % segments;
            triangles.push([0, bottom_rim + next, bottom_rim + i]);
            triangles.push([top_center, top_rim + i, top_rim + next]);
            triangles.push([bottom_rim + i, bottom_rim + next, top_rim + next]);
            triangles.push([bottom_rim + i, top_rim + next, top_rim + i]);
        }

        Mesh::new(vertices, triangles)
    }
}

fn rim_point(radius: f64, z: f64, i: usize, segments: usize) -> Point3 {
    let theta = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
    Point3::new(radius * theta.cos(), radius * theta.sin(), z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::solid::circle_segments;

    #[test]
    fn mesh_segment_count_follows_circumference() {
        let segments = circle_segments(10.0, 1.0);
        let mesh = Cylinder::new(10.mm(), 20.mm()).mesh(1.mm());

        assert_eq!(segments, 63);
        assert_eq!(mesh.vertices.len(), 2 * segments + 2);
        assert_eq!(mesh.triangles.len(), 4 * segments);
    }

    #[test]
    fn coarser_tolerance_uses_fewer_segments() {
        let fine = Cylinder::new(10.mm(), 20.mm()).mesh(1.mm());
        let coarse = Cylinder::new(10.mm(), 20.mm()).mesh(5.mm());

        assert!(coarse.vertices.len() < fine.vertices.len());
        assert!(coarse.triangles.len() < fine.triangles.len());
    }

    #[test]
    fn mesh_is_centered_with_rims_on_the_radius() {
        let mesh = Cylinder::new(10.mm(), 20.mm()).mesh(1.mm());
        let segments = (mesh.vertices.len() - 2) / 2;
        let half_height = 10.0;

        assert_eq!(mesh.vertices[0], Point3::new(0.0, 0.0, -half_height));
        assert_eq!(
            mesh.vertices[segments + 1],
            Point3::new(0.0, 0.0, half_height)
        );

        for vertex in mesh.vertices[1..=segments]
            .iter()
            .chain(mesh.vertices[segments + 2..].iter())
        {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y).sqrt();
            assert!((radius - 10.0).abs() < 1e-9);
            assert!((vertex.z.abs() - half_height).abs() < 1e-9);
        }
    }

    #[test]
    fn side_triangles_face_outward() {
        let mesh = Cylinder::new(10.mm(), 20.mm()).mesh(1.mm());
        let [a, b, c] = mesh.triangles[2];
        let ab = (
            mesh.vertices[b].x - mesh.vertices[a].x,
            mesh.vertices[b].y - mesh.vertices[a].y,
            mesh.vertices[b].z - mesh.vertices[a].z,
        );
        let ac = (
            mesh.vertices[c].x - mesh.vertices[a].x,
            mesh.vertices[c].y - mesh.vertices[a].y,
            mesh.vertices[c].z - mesh.vertices[a].z,
        );
        let nx = ab.1 * ac.2 - ab.2 * ac.1;

        assert!(nx > 0.0);
    }
}
