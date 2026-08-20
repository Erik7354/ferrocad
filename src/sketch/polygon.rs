use crate::{
    length::Length,
    mesh::Point2,
    sketch::{Sketch, triangulate::twice_signed_area},
};

/// Closed polygon in the XY plane of the sketch.
///
/// The vertices specify the shape in sketch coordinates. The polygon is not
/// centered on the origin.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub points: Vec<[Length; 2]>,
}

impl Polygon {
    pub fn new(points: Vec<[Length; 2]>) -> Self {
        Self { points }
    }
}

impl Sketch for Polygon {
    fn contour(&self, _tolerance: Length) -> Vec<Point2> {
        let mut points = self.points.clone();
        if points.len() >= 2 && points.first() == points.last() {
            points.pop();
        }
        points.dedup();
        if points.len() < 3 {
            return Vec::new();
        }

        let mut contour: Vec<Point2> = points
            .iter()
            .map(|[x, y]| Point2::new(x.as_mm_f64(), y.as_mm_f64()))
            .collect();
        if twice_signed_area(&contour) < 0.0 {
            contour.reverse();
        }
        contour
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::mesh::Point2;
    use crate::sketch::triangulate::twice_signed_area;
    use crate::solid::Solid;
    use crate::transform::Pose2;

    fn c_polygon() -> Polygon {
        Polygon::new(vec![
            [0.mm(), 0.mm()],
            [10.mm(), 0.mm()],
            [10.mm(), 2.mm()],
            [2.mm(), 2.mm()],
            [2.mm(), 8.mm()],
            [10.mm(), 8.mm()],
            [10.mm(), 10.mm()],
            [0.mm(), 10.mm()],
        ])
    }

    #[test]
    fn contour_keeps_authored_vertices() {
        let contour = Polygon::new(vec![[0.mm(), 0.mm()], [10.mm(), 0.mm()], [0.mm(), 5.mm()]])
            .contour(1.mm());

        assert_eq!(
            contour,
            vec![
                Point2::new(0.0, 0.0),
                Point2::new(10.0, 0.0),
                Point2::new(0.0, 5.0),
            ]
        );
    }

    #[test]
    fn contour_drops_a_repeated_closing_vertex() {
        let contour = Polygon::new(vec![
            [0.mm(), 0.mm()],
            [10.mm(), 0.mm()],
            [0.mm(), 5.mm()],
            [0.mm(), 0.mm()],
        ])
        .contour(1.mm());

        assert_eq!(contour.len(), 3);
        assert_eq!(contour[0], Point2::new(0.0, 0.0));
        assert_eq!(contour[2], Point2::new(0.0, 5.0));
    }

    #[test]
    fn clockwise_input_becomes_counterclockwise() {
        let contour = Polygon::new(vec![[0.mm(), 0.mm()], [0.mm(), 10.mm()], [10.mm(), 0.mm()]])
            .contour(1.mm());

        assert!(twice_signed_area(&contour) > 0.0);
        assert_eq!(contour[0], Point2::new(10.0, 0.0));
        assert_eq!(contour[1], Point2::new(0.0, 10.0));
        assert_eq!(contour[2], Point2::new(0.0, 0.0));
    }

    #[test]
    fn fewer_than_three_points_yields_an_empty_contour() {
        assert!(
            Polygon::new(vec![[0.mm(), 0.mm()], [1.mm(), 0.mm()]])
                .contour(1.mm())
                .is_empty()
        );
        assert!(
            Polygon::new(vec![[0.mm(), 0.mm()], [1.mm(), 0.mm()], [0.mm(), 0.mm()],])
                .contour(1.mm())
                .is_empty()
        );
    }

    #[test]
    fn extrude_keeps_the_polygon_and_height() {
        let polygon = c_polygon();
        let extrusion = polygon.clone().extrude(8.mm());

        assert_eq!(extrusion.sketch, polygon);
        assert_eq!(extrusion.height, 8.mm());
    }

    #[test]
    fn convex_polygon_volume_matches_the_prism() {
        let mesh = Polygon::new(vec![
            [-10.mm(), -5.mm()],
            [10.mm(), -5.mm()],
            [10.mm(), 5.mm()],
            [-10.mm(), 5.mm()],
        ])
        .extrude(8.mm())
        .mesh(1.mm());

        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.triangles.len(), 12);
        assert!((mesh.signed_volume() - 1600.0).abs() < 1e-9);
    }

    #[test]
    fn concave_c_extrusion_has_the_prism_volume() {
        let mesh = c_polygon().extrude(8.mm()).mesh(1.mm());
        let n = 8;
        assert_eq!(mesh.vertices.len(), 2 * n);
        assert_eq!(mesh.triangles.len(), 4 * n - 4);
        assert!(mesh.signed_volume() > 0.0);
        assert!((mesh.signed_volume() - 416.0).abs() < 1e-9);
    }

    #[test]
    fn translated_polygon_shifts_the_contour() {
        let polygon = c_polygon();
        let original = polygon.contour(1.mm());
        let contour = polygon.translate(4.mm(), -1.mm()).contour(1.mm());

        assert_eq!(contour.len(), original.len());
        for (translated, original) in contour.iter().zip(&original) {
            assert_eq!(*translated, Point2::new(original.x + 4.0, original.y - 1.0));
        }
    }

    #[test]
    fn mirror_keeps_a_polygon_contour_counterclockwise() {
        let contour = Polygon::new(vec![
            [0.mm(), 0.mm()],
            [10.mm(), 0.mm()],
            [10.mm(), 5.mm()],
            [0.mm(), 5.mm()],
        ])
        .mirror(1.0, 0.0)
        .contour(1.mm());

        assert!(twice_signed_area(&contour) > 0.0);
        assert_eq!(
            contour,
            vec![
                Point2::new(0.0, 5.0),
                Point2::new(-10.0, 5.0),
                Point2::new(-10.0, 0.0),
                Point2::new(0.0, 0.0),
            ]
        );
    }
}
