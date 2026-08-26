use std::f64::consts::PI;

use crate::{
    angle::Angle,
    length::Length,
    mesh::{Mesh, Point3},
    sketch::{Sketch, triangulate::triangulate},
};

use super::{Solid, chord_segments};

/// A solid produced by lifting a [`Sketch`] along Z with a rotation around Z.
///
/// The solid is centered on the XY plane. The bottom face is the sketch. The
/// top face is the sketch after rotation by [`twist`](Self::twist) around the
/// Z-axis. A positive twist is counterclockwise around +Z.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistExtrusion<S> {
    pub sketch: S,
    pub height: Length,
    pub twist: Angle,
}

impl<S> TwistExtrusion<S> {
    pub const fn new(sketch: S, height: Length, twist: Angle) -> Self {
        Self {
            sketch,
            height,
            twist,
        }
    }
}

impl<S: Sketch> Solid for TwistExtrusion<S> {
    fn mesh(&self, tolerance: Length) -> Mesh {
        let contour = self.sketch.contour(tolerance);
        let n = contour.len();
        if n < 3 {
            return Mesh::new(Vec::new(), Vec::new());
        }

        let height_mm = self.height.as_mm_f64();
        let half_height = height_mm / 2.0;
        let twist_rad = self.twist.as_rad();
        let r_max = contour
            .iter()
            .map(|point| (point.x * point.x + point.y * point.y).sqrt())
            .fold(0.0, f64::max);
        let turns = (twist_rad.abs() / (2.0 * PI)).ceil().max(1.0);
        let max_slices = (360.0 * turns) as usize;
        let slices = chord_segments(
            r_max * twist_rad.abs(),
            tolerance.as_mm_f64(),
            1,
            max_slices,
        );
        let ring_count = slices + 1;

        let mut vertices = Vec::with_capacity(ring_count * n);
        for k in 0..ring_count {
            let t = k as f64 / slices as f64;
            let z = -half_height + t * height_mm;
            let theta = t * twist_rad;
            let (cos_t, sin_t) = (theta.cos(), theta.sin());
            for point in &contour {
                vertices.push(Point3::new(
                    point.x * cos_t - point.y * sin_t,
                    point.x * sin_t + point.y * cos_t,
                    z,
                ));
            }
        }

        let caps = triangulate(&contour);
        if caps.is_empty() {
            return Mesh::new(Vec::new(), Vec::new());
        }

        let last = slices * n;
        let mut triangles = Vec::with_capacity(2 * (n - 2) + 2 * n * slices);
        for &[a, b, c] in &caps {
            triangles.push([a, c, b]);
            triangles.push([last + a, last + b, last + c]);
        }
        for r in 0..slices {
            let r0 = r * n;
            let r1 = (r + 1) * n;
            for i in 0..n {
                let next = (i + 1) % n;
                triangles.push([r0 + i, r0 + next, r1 + next]);
                triangles.push([r0 + i, r1 + next, r1 + i]);
            }
        }

        Mesh::new(vertices, triangles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToAngle;
    use crate::ToLength;
    use crate::mesh::Point2;
    use crate::sketch::{Circle, Rectangle};
    use crate::solid::Solid;

    fn assert_faces_outward(mesh: &Mesh, interior: Point3) {
        for &[a, b, c] in &mesh.triangles {
            let pa = mesh.vertices[a];
            let pb = mesh.vertices[b];
            let pc = mesh.vertices[c];
            let ab = (pb.x - pa.x, pb.y - pa.y, pb.z - pa.z);
            let ac = (pc.x - pa.x, pc.y - pa.y, pc.z - pa.z);
            let normal = (
                ab.1 * ac.2 - ab.2 * ac.1,
                ab.2 * ac.0 - ab.0 * ac.2,
                ab.0 * ac.1 - ab.1 * ac.0,
            );
            let centroid = (
                (pa.x + pb.x + pc.x) / 3.0,
                (pa.y + pb.y + pc.y) / 3.0,
                (pa.z + pb.z + pc.z) / 3.0,
            );
            let offset = (
                centroid.0 - interior.x,
                centroid.1 - interior.y,
                centroid.2 - interior.z,
            );
            assert!(normal.0 * offset.0 + normal.1 * offset.1 + normal.2 * offset.2 > 0.0);
        }
    }

    fn assert_point_near(got: Point3, expected: Point3) {
        assert!((got.x - expected.x).abs() < 1e-9);
        assert!((got.y - expected.y).abs() < 1e-9);
        assert!((got.z - expected.z).abs() < 1e-9);
    }

    #[test]
    fn twist_extrude_keeps_the_sketch_height_and_twist() {
        let rectangle = Rectangle::new(20.mm(), 10.mm());
        let solid = rectangle.twist_extrude(8.mm(), 90.deg());

        assert_eq!(solid.sketch, rectangle);
        assert_eq!(solid.height, 8.mm());
        assert_eq!(solid.twist, 90.deg());
    }

    #[test]
    fn empty_contour_yields_an_empty_mesh() {
        struct Empty;

        impl Sketch for Empty {
            fn contour(&self, _tolerance: Length) -> Vec<Point2> {
                Vec::new()
            }
        }

        let mesh = TwistExtrusion::new(Empty, 10.mm(), 90.deg()).mesh(1.mm());
        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn short_contour_yields_an_empty_mesh() {
        struct Short;

        impl Sketch for Short {
            fn contour(&self, _tolerance: Length) -> Vec<Point2> {
                vec![Point2::new(1.0, 0.0), Point2::new(2.0, 0.0)]
            }
        }

        let mesh = TwistExtrusion::new(Short, 10.mm(), 90.deg()).mesh(1.mm());
        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn zero_twist_matches_extrude() {
        let extruded = Rectangle::new(20.mm(), 10.mm())
            .extrude(8.mm())
            .mesh(1.mm());
        let twisted = Rectangle::new(20.mm(), 10.mm())
            .twist_extrude(8.mm(), Angle::ZERO)
            .mesh(1.mm());

        assert_eq!(extruded.vertices, twisted.vertices);
        assert_eq!(extruded.triangles, twisted.triangles);
    }

    #[test]
    fn bottom_ring_matches_the_sketch_and_top_ring_is_rotated() {
        let mesh = Rectangle::new(20.mm(), 10.mm())
            .twist_extrude(8.mm(), 90.deg())
            .mesh(1.mm());
        let n = 4;
        let last = mesh.vertices.len() - n;

        assert_eq!(mesh.vertices[0], Point3::new(-10.0, -5.0, -4.0));
        assert_eq!(mesh.vertices[1], Point3::new(10.0, -5.0, -4.0));
        assert_eq!(mesh.vertices[2], Point3::new(10.0, 5.0, -4.0));
        assert_eq!(mesh.vertices[3], Point3::new(-10.0, 5.0, -4.0));
        assert_point_near(mesh.vertices[last], Point3::new(5.0, -10.0, 4.0));
        assert_point_near(mesh.vertices[last + 1], Point3::new(5.0, 10.0, 4.0));
        assert_point_near(mesh.vertices[last + 2], Point3::new(-5.0, 10.0, 4.0));
        assert_point_near(mesh.vertices[last + 3], Point3::new(-5.0, -10.0, 4.0));
    }

    #[test]
    fn circle_twist_stays_a_cylinder() {
        let mesh = Circle::new(10.mm())
            .twist_extrude(20.mm(), 90.deg())
            .mesh(1.mm());
        let expected = PI * 100.0 * 20.0;

        for vertex in &mesh.vertices {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y).sqrt();
            assert!((radius - 10.0).abs() < 1e-9);
        }
        assert!(mesh.signed_volume() > 0.0);
        assert!((mesh.signed_volume() - expected).abs() / expected < 0.02);
    }

    #[test]
    fn rectangle_volume_matches_the_prism() {
        let mesh = Rectangle::new(20.mm(), 10.mm())
            .twist_extrude(8.mm(), 90.deg())
            .mesh(0.2.mm());
        let expected = 20.0 * 10.0 * 8.0;
        let volume = mesh.signed_volume();

        assert!(volume > 0.0);
        assert!((volume - expected).abs() / expected < 0.02);
    }

    #[test]
    fn coarser_tolerance_uses_fewer_rings() {
        let fine = Circle::new(10.mm())
            .twist_extrude(20.mm(), 360.deg())
            .mesh(1.mm());
        let coarse = Circle::new(10.mm())
            .twist_extrude(20.mm(), 360.deg())
            .mesh(5.mm());

        assert!(coarse.vertices.len() < fine.vertices.len());
        assert!(coarse.triangles.len() < fine.triangles.len());
    }

    #[test]
    fn twisted_mesh_faces_outward_and_has_positive_volume() {
        let mesh = Circle::new(10.mm())
            .twist_extrude(20.mm(), 90.deg())
            .mesh(1.mm());

        assert!(mesh.signed_volume() > 0.0);
        assert_faces_outward(&mesh, Point3::new(0.0, 0.0, 0.0));
    }
}
