use std::f64::consts::PI;

use crate::{
    angle::Angle,
    length::Length,
    mesh::{Mesh, Point3},
    sketch::{Sketch, triangulate::triangulate},
};

use super::{Solid, chord_segments, circle_segments};

const AXIS_EPS: f64 = 1e-9;

/// A solid produced by rotation of a [`Sketch`] around the Z-axis.
///
/// The X-coordinate of the sketch is the radius. The Y-coordinate of the
/// sketch is the Z-coordinate of the solid. Each point of the sketch must
/// have an X-coordinate that is zero or more.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotateExtrusion<S> {
    pub sketch: S,
    pub angle: Angle,
}

impl<S> RotateExtrusion<S> {
    pub const fn new(sketch: S, angle: Angle) -> Self {
        Self { sketch, angle }
    }
}

impl<S: Sketch> Solid for RotateExtrusion<S> {
    fn mesh(&self, tolerance: Length) -> Mesh {
        let mut contour = self.sketch.contour(tolerance);
        let n = contour.len();
        if n < 3 {
            return Mesh::new(Vec::new(), Vec::new());
        }

        for point in &mut contour {
            if point.x < -AXIS_EPS {
                return Mesh::new(Vec::new(), Vec::new());
            }
            if point.x < 0.0 {
                point.x = 0.0;
            }
        }

        let angle_rad = self.angle.as_rad();
        if angle_rad.abs() < AXIS_EPS {
            return Mesh::new(Vec::new(), Vec::new());
        }

        let closed = angle_rad.abs() + AXIS_EPS >= 2.0 * PI;
        let sweep = if closed {
            if angle_rad >= 0.0 {
                2.0 * PI
            } else {
                -2.0 * PI
            }
        } else {
            angle_rad
        };

        let r_max = contour.iter().map(|point| point.x).fold(0.0, f64::max);
        if r_max <= AXIS_EPS {
            return Mesh::new(Vec::new(), Vec::new());
        }

        let tolerance_mm = tolerance.as_mm_f64();
        let (ring_count, span) = if closed {
            let steps = circle_segments(r_max, tolerance_mm);
            (steps, steps)
        } else {
            let steps = chord_segments(sweep.abs() * r_max, tolerance_mm, 1, 360);
            (steps + 1, steps)
        };

        let mut vertices = Vec::with_capacity(ring_count * n);
        for k in 0..ring_count {
            let theta = sweep * k as f64 / span as f64;
            let (cos_t, sin_t) = (theta.cos(), theta.sin());
            for point in &contour {
                vertices.push(Point3::new(point.x * cos_t, point.x * sin_t, point.y));
            }
        }

        let flip = sweep < 0.0;
        let mut triangles = Vec::new();
        let ring_pairs = if closed { ring_count } else { ring_count - 1 };
        for r in 0..ring_pairs {
            let r0 = r;
            let r1 = (r + 1) % ring_count;
            for i in 0..n {
                let j = (i + 1) % n;
                push_side(
                    &mut triangles,
                    r0 * n + i,
                    r0 * n + j,
                    r1 * n + i,
                    r1 * n + j,
                    contour[i].x <= AXIS_EPS,
                    contour[j].x <= AXIS_EPS,
                    flip,
                );
            }
        }

        if !closed {
            let caps = triangulate(&contour);
            if caps.is_empty() {
                return Mesh::new(Vec::new(), Vec::new());
            }
            let last = (ring_count - 1) * n;
            for &[a, b, c] in &caps {
                if flip {
                    triangles.push([a, c, b]);
                    triangles.push([last + a, last + b, last + c]);
                } else {
                    triangles.push([a, b, c]);
                    triangles.push([last + a, last + c, last + b]);
                }
            }
        }

        Mesh::new(vertices, triangles)
    }
}

fn push_side(
    triangles: &mut Vec<[usize; 3]>,
    i0: usize,
    j0: usize,
    i1: usize,
    j1: usize,
    i_on_axis: bool,
    j_on_axis: bool,
    flip: bool,
) {
    if i_on_axis && j_on_axis {
        return;
    }
    if i_on_axis {
        if flip {
            triangles.push([i0, j0, j1]);
        } else {
            triangles.push([i0, j1, j0]);
        }
    } else if j_on_axis {
        if flip {
            triangles.push([i0, j0, i1]);
        } else {
            triangles.push([i0, i1, j1]);
        }
    } else if flip {
        triangles.push([i0, j0, j1]);
        triangles.push([i0, j1, i1]);
    } else {
        triangles.push([i0, j1, j0]);
        triangles.push([i0, i1, j1]);
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
    use crate::transform::Pose2;

    fn cylinder_profile() -> impl Sketch {
        Rectangle::new(10.mm(), 20.mm()).translate(5.mm(), Length::ZERO)
    }

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

    #[test]
    fn rotate_extrude_keeps_the_sketch_and_angle() {
        let circle = Circle::new(5.mm());
        let solid = circle.rotate_extrude(90.deg());

        assert_eq!(solid.sketch, circle);
        assert_eq!(solid.angle, 90.deg());
    }

    #[test]
    fn full_rectangle_volume_matches_a_prism_cylinder() {
        let mesh = cylinder_profile().rotate_extrude(360.deg()).mesh(1.mm());
        let steps = circle_segments(10.0, 1.0);
        let area = steps as f64 / 2.0 * 100.0 * (2.0 * PI / steps as f64).sin();
        let expected = area * 20.0;

        assert!(mesh.signed_volume() > 0.0);
        assert!((mesh.signed_volume() - expected).abs() < 1e-6);
    }

    #[test]
    fn torus_volume_matches_the_ring() {
        let mesh = Circle::new(5.mm())
            .translate(20.mm(), Length::ZERO)
            .rotate_extrude(360.deg())
            .mesh(1.mm());
        let expected = 2.0 * PI * PI * 20.0 * 25.0;

        assert!(mesh.signed_volume() > 0.0);
        assert!((mesh.signed_volume() - expected).abs() / expected < 0.02);
    }

    #[test]
    fn partial_rectangle_is_a_quarter_of_the_full_volume() {
        let full = cylinder_profile()
            .rotate_extrude(360.deg())
            .mesh(1.mm())
            .signed_volume();
        let quarter = cylinder_profile()
            .rotate_extrude(90.deg())
            .mesh(1.mm())
            .signed_volume();

        assert!(quarter > 0.0);
        assert!((quarter - full / 4.0).abs() / (full / 4.0) < 0.02);
    }

    #[test]
    fn negative_x_contour_yields_an_empty_mesh() {
        let mesh = Rectangle::new(10.mm(), 20.mm())
            .rotate_extrude(360.deg())
            .mesh(1.mm());
        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn empty_contour_yields_an_empty_mesh() {
        struct Empty;

        impl Sketch for Empty {
            fn contour(&self, _tolerance: Length) -> Vec<Point2> {
                Vec::new()
            }
        }

        let mesh = RotateExtrusion::new(Empty, 360.deg()).mesh(1.mm());
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

        let mesh = RotateExtrusion::new(Short, 360.deg()).mesh(1.mm());
        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn zero_angle_yields_an_empty_mesh() {
        let mesh = cylinder_profile().rotate_extrude(Angle::ZERO).mesh(1.mm());
        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn coarser_tolerance_uses_fewer_segments() {
        let fine = Circle::new(5.mm())
            .translate(20.mm(), Length::ZERO)
            .rotate_extrude(360.deg())
            .mesh(1.mm());
        let coarse = Circle::new(5.mm())
            .translate(20.mm(), Length::ZERO)
            .rotate_extrude(360.deg())
            .mesh(5.mm());

        assert!(coarse.vertices.len() < fine.vertices.len());
        assert!(coarse.triangles.len() < fine.triangles.len());
    }

    #[test]
    fn revolved_mesh_faces_outward_and_has_positive_volume() {
        let mesh = cylinder_profile().rotate_extrude(360.deg()).mesh(1.mm());

        assert!(mesh.signed_volume() > 0.0);
        assert_faces_outward(&mesh, Point3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn partial_mesh_faces_outward() {
        let mesh = cylinder_profile().rotate_extrude(90.deg()).mesh(1.mm());

        assert!(mesh.signed_volume() > 0.0);
        assert_faces_outward(&mesh, Point3::new(5.0, 5.0, 0.0));
    }
}
