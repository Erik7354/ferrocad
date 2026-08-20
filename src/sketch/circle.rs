use crate::{length::Length, mesh::Point2, sketch::Sketch, solid::circle_segments};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub radius: Length,
}

impl Circle {
    pub const fn new(radius: Length) -> Self {
        Self { radius }
    }

    pub const fn diameter(diameter: Length) -> Self {
        Self::new(Length::um(diameter.as_um() / 2))
    }
}

impl Sketch for Circle {
    fn contour(&self, tolerance: Length) -> Vec<Point2> {
        let radius = self.radius.as_mm_f64();
        let segments = circle_segments(radius, tolerance.as_mm_f64());
        (0..segments)
            .map(|i| {
                let theta = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
                Point2::new(radius * theta.cos(), radius * theta.sin())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::solid::{Solid, circle_segments};

    #[test]
    fn diameter_is_twice_the_radius() {
        assert_eq!(Circle::diameter(20.mm()), Circle::new(10.mm()));
        assert_eq!(Circle::diameter(5.mm()), Circle::new(2_500.um()));
    }

    #[test]
    fn contour_points_lie_on_the_radius() {
        let segments = circle_segments(10.0, 1.0);
        let contour = Circle::new(10.mm()).contour(1.mm());

        assert_eq!(contour.len(), segments);
        for point in &contour {
            let radius = (point.x * point.x + point.y * point.y).sqrt();
            assert!((radius - 10.0).abs() < 1e-9);
        }
    }

    #[test]
    fn contour_starts_on_the_positive_x_axis_and_winds_counterclockwise() {
        let contour = Circle::new(10.mm()).contour(1.mm());

        assert!((contour[0].x - 10.0).abs() < 1e-9);
        assert!(contour[0].y.abs() < 1e-9);
        assert!(contour[1].y > 0.0);
    }

    #[test]
    fn extrude_keeps_the_circle_and_height() {
        let circle = Circle::new(10.mm());
        let extrusion = circle.extrude(20.mm());

        assert_eq!(extrusion.sketch, circle);
        assert_eq!(extrusion.height, 20.mm());
    }

    #[test]
    fn extruded_mesh_segment_count_follows_circumference() {
        let segments = circle_segments(10.0, 1.0);
        let mesh = Circle::new(10.mm()).extrude(20.mm()).mesh(1.mm());

        assert_eq!(segments, 63);
        assert_eq!(mesh.vertices.len(), 2 * segments);
        assert_eq!(mesh.triangles.len(), 4 * segments - 4);
    }

    #[test]
    fn coarser_tolerance_uses_fewer_segments() {
        let fine = Circle::new(10.mm()).extrude(20.mm()).mesh(1.mm());
        let coarse = Circle::new(10.mm()).extrude(20.mm()).mesh(5.mm());

        assert!(coarse.vertices.len() < fine.vertices.len());
        assert!(coarse.triangles.len() < fine.triangles.len());
    }
}
