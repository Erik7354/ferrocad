use crate::{
    length::Length,
    mesh::{Mesh, Point3},
    solid::Solid,
};

/// A solid cylinder along the Z-axis. The solid is centered on the origin.
///
/// Use [`Self::frustum`] when the two end radii are different. Use [`Self::cone`]
/// when the top end is a point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cylinder {
    pub bottom_radius: Length,
    pub top_radius: Length,
    pub height: Length,
}

impl Cylinder {
    pub const fn new(radius: Length, height: Length) -> Self {
        Self {
            bottom_radius: radius,
            top_radius: radius,
            height,
        }
    }

    pub const fn frustum(bottom_radius: Length, top_radius: Length, height: Length) -> Self {
        Self {
            bottom_radius,
            top_radius,
            height,
        }
    }

    pub const fn cone(radius: Length, height: Length) -> Self {
        Self::frustum(radius, Length::ZERO, height)
    }
}

impl Solid for Cylinder {
    fn mesh(&self, tolerance: Length) -> Mesh {
        let bottom_radius = self.bottom_radius.as_mm_f64().max(0.0);
        let top_radius = self.top_radius.as_mm_f64().max(0.0);
        if bottom_radius == 0.0 && top_radius == 0.0 {
            return Mesh::new(Vec::new(), Vec::new());
        }

        let half_height = self.height.as_mm_f64() / 2.0;
        let segments = super::circle_segments(bottom_radius.max(top_radius), tolerance.as_mm_f64());

        let mut vertices = Vec::with_capacity(2 + 2 * segments);
        vertices.push(Point3::new(0.0, 0.0, -half_height));
        if bottom_radius > 0.0 {
            for i in 0..segments {
                vertices.push(rim_point(bottom_radius, -half_height, i, segments));
            }
        }
        vertices.push(Point3::new(0.0, 0.0, half_height));
        if top_radius > 0.0 {
            for i in 0..segments {
                vertices.push(rim_point(top_radius, half_height, i, segments));
            }
        }

        let bottom_center = 0;
        let (bottom_rim, top_center) = if bottom_radius > 0.0 {
            (Some(1), 1 + segments)
        } else {
            (None, 1)
        };
        let top_rim = if top_radius > 0.0 {
            Some(top_center + 1)
        } else {
            None
        };

        let mut triangles = Vec::with_capacity(4 * segments);
        for i in 0..segments {
            let next = (i + 1) % segments;
            if let Some(br) = bottom_rim {
                triangles.push([bottom_center, br + next, br + i]);
            }
            if let Some(tr) = top_rim {
                triangles.push([top_center, tr + i, tr + next]);
            }
            match (bottom_rim, top_rim) {
                (Some(br), Some(tr)) => {
                    triangles.push([br + i, br + next, tr + next]);
                    triangles.push([br + i, tr + next, tr + i]);
                }
                (Some(br), None) => {
                    triangles.push([br + i, br + next, top_center]);
                }
                (None, Some(tr)) => {
                    triangles.push([bottom_center, tr + next, tr + i]);
                }
                (None, None) => {}
            }
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
    fn frustum_segment_count_follows_the_larger_radius() {
        let segments = circle_segments(20.0, 1.0);
        let mesh = Cylinder::frustum(5.mm(), 20.mm(), 10.mm()).mesh(1.mm());

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
    fn frustum_rims_use_the_two_radii() {
        let mesh = Cylinder::frustum(5.mm(), 15.mm(), 20.mm()).mesh(1.mm());
        let segments = (mesh.vertices.len() - 2) / 2;

        for vertex in &mesh.vertices[1..=segments] {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y).sqrt();
            assert!((radius - 5.0).abs() < 1e-9);
            assert!((vertex.z + 10.0).abs() < 1e-9);
        }
        for vertex in &mesh.vertices[segments + 2..] {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y).sqrt();
            assert!((radius - 15.0).abs() < 1e-9);
            assert!((vertex.z - 10.0).abs() < 1e-9);
        }
    }

    #[test]
    fn cone_collapses_the_top_to_an_apex() {
        let segments = circle_segments(10.0, 1.0);
        let mesh = Cylinder::cone(10.mm(), 20.mm()).mesh(1.mm());

        assert_eq!(mesh.vertices.len(), segments + 2);
        assert_eq!(mesh.triangles.len(), 2 * segments);
        assert_eq!(mesh.vertices[0], Point3::new(0.0, 0.0, -10.0));
        assert_eq!(mesh.vertices[segments + 1], Point3::new(0.0, 0.0, 10.0));

        for vertex in &mesh.vertices[1..=segments] {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y).sqrt();
            assert!((radius - 10.0).abs() < 1e-9);
            assert!((vertex.z + 10.0).abs() < 1e-9);
        }
    }

    #[test]
    fn inverted_cone_collapses_the_bottom_to_an_apex() {
        let segments = circle_segments(10.0, 1.0);
        let mesh = Cylinder::frustum(Length::ZERO, 10.mm(), 20.mm()).mesh(1.mm());

        assert_eq!(mesh.vertices.len(), segments + 2);
        assert_eq!(mesh.triangles.len(), 2 * segments);
        assert_eq!(mesh.vertices[0], Point3::new(0.0, 0.0, -10.0));
        assert_eq!(mesh.vertices[1], Point3::new(0.0, 0.0, 10.0));
    }

    #[test]
    fn both_radii_zero_yields_an_empty_mesh() {
        let mesh = Cylinder::frustum(Length::ZERO, Length::ZERO, 20.mm()).mesh(1.mm());
        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
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

    #[test]
    fn cone_side_triangles_face_outward() {
        let mesh = Cylinder::cone(10.mm(), 20.mm()).mesh(1.mm());
        let [a, b, c] = mesh.triangles[1];
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
