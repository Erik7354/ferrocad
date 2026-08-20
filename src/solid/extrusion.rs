use crate::{
    length::Length,
    mesh::{Mesh, Point3},
    sketch::Sketch,
};

use super::Solid;

/// A solid produced by lifting a [`Sketch`] along Z.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extrusion<S> {
    pub sketch: S,
    pub height: Length,
}

impl<S> Extrusion<S> {
    pub const fn new(sketch: S, height: Length) -> Self {
        Self { sketch, height }
    }
}

impl<S: Sketch> Solid for Extrusion<S> {
    fn mesh(&self, tolerance: Length) -> Mesh {
        let contour = self.sketch.contour(tolerance);
        let n = contour.len();
        if n < 3 {
            return Mesh::new(Vec::new(), Vec::new());
        }

        let half_height = self.height.as_mm_f64() / 2.0;
        let mut vertices = Vec::with_capacity(2 * n);
        for point in &contour {
            vertices.push(Point3::new(point.x, point.y, -half_height));
        }
        for point in &contour {
            vertices.push(Point3::new(point.x, point.y, half_height));
        }

        let mut triangles = Vec::with_capacity(4 * n - 4);
        for i in 1..n - 1 {
            triangles.push([0, i + 1, i]);
            triangles.push([n, n + i, n + i + 1]);
        }
        for i in 0..n {
            let next = (i + 1) % n;
            triangles.push([i, next, n + next]);
            triangles.push([i, n + next, n + i]);
        }

        Mesh::new(vertices, triangles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::mesh::Point3;
    use crate::sketch::Circle;
    use crate::solid::Solid;

    #[test]
    fn mesh_is_centered_with_rims_on_the_sketch() {
        let mesh = Circle::new(10.mm()).extrude(20.mm()).mesh(1.mm());
        let segments = mesh.vertices.len() / 2;
        let half_height = 10.0;

        for vertex in mesh.vertices[..segments].iter() {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y).sqrt();
            assert!((radius - 10.0).abs() < 1e-9);
            assert!((vertex.z + half_height).abs() < 1e-9);
        }
        for vertex in mesh.vertices[segments..].iter() {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y).sqrt();
            assert!((radius - 10.0).abs() < 1e-9);
            assert!((vertex.z - half_height).abs() < 1e-9);
        }
    }

    #[test]
    fn side_triangles_face_outward() {
        let mesh = Circle::new(10.mm()).extrude(20.mm()).mesh(1.mm());
        let n = mesh.vertices.len() / 2;
        let [a, b, c] = mesh.triangles[2 * (n - 2)];
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
    fn empty_contour_yields_an_empty_mesh() {
        struct Empty;

        impl Sketch for Empty {
            fn contour(&self, _tolerance: Length) -> Vec<crate::mesh::Point2> {
                Vec::new()
            }
        }

        let mesh = Extrusion::new(Empty, 10.mm()).mesh(1.mm());
        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn bottom_and_top_caps_use_the_first_contour_point() {
        let mesh = Circle::new(10.mm()).extrude(20.mm()).mesh(1.mm());
        let n = mesh.vertices.len() / 2;

        assert_eq!(mesh.vertices[0], Point3::new(10.0, 0.0, -10.0));
        assert_eq!(mesh.vertices[n], Point3::new(10.0, 0.0, 10.0));
        assert_eq!(mesh.triangles[0], [0, 2, 1]);
        assert_eq!(mesh.triangles[1], [n, n + 1, n + 2]);
    }
}
