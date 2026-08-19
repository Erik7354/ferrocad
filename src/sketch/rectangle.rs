use crate::{length::Length, mesh::Point2, sketch::Sketch};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub width: Length,
    pub depth: Length,
}

impl Rectangle {
    pub const fn new(width: Length, depth: Length) -> Self {
        Self { width, depth }
    }

    pub const fn square(size: Length) -> Self {
        Self::new(size, size)
    }
}

impl Sketch for Rectangle {
    fn contour(&self, _tolerance: Length) -> Vec<Point2> {
        let half_width = self.width.as_mm_f64() / 2.0;
        let half_depth = self.depth.as_mm_f64() / 2.0;
        vec![
            Point2::new(-half_width, -half_depth),
            Point2::new(half_width, -half_depth),
            Point2::new(half_width, half_depth),
            Point2::new(-half_width, half_depth),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::solid::{Cuboid, Solid};

    #[test]
    fn contour_is_a_centered_rectangle() {
        let contour = Rectangle::new(20.mm(), 10.mm()).contour(1.mm());

        assert_eq!(
            contour,
            vec![
                Point2::new(-10.0, -5.0),
                Point2::new(10.0, -5.0),
                Point2::new(10.0, 5.0),
                Point2::new(-10.0, 5.0),
            ]
        );
    }

    #[test]
    fn square_is_a_rectangle_with_equal_sides() {
        assert_eq!(Rectangle::square(20.mm()), Rectangle::new(20.mm(), 20.mm()));
    }

    #[test]
    fn extrude_keeps_the_rectangle_and_height() {
        let rectangle = Rectangle::new(20.mm(), 10.mm());
        let extrusion = rectangle.extrude(8.mm());

        assert_eq!(extrusion.sketch, rectangle);
        assert_eq!(extrusion.height, 8.mm());
    }

    #[test]
    fn extruded_vertices_match_a_cuboid() {
        let extruded = Rectangle::new(20.mm(), 10.mm())
            .extrude(8.mm())
            .mesh(1.mm());
        let cuboid = Cuboid::new(20.mm(), 10.mm(), 8.mm()).mesh(1.mm());

        assert_eq!(extruded.vertices, cuboid.vertices);
        assert_eq!(extruded.triangles.len(), cuboid.triangles.len());
    }
}
