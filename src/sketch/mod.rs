use crate::angle::Angle;
use crate::length::Length;
use crate::mesh::Point2;
use crate::solid::{Extrusion, RotateExtrusion, TwistExtrusion};

pub mod circle;
pub mod polygon;
pub mod rectangle;
pub(crate) mod triangulate;

pub use circle::Circle;
pub use polygon::Polygon;
pub use rectangle::Rectangle;

/// A Sketch is a flat shape in the XY plane.
pub trait Sketch: Sized {
    /// Closed contour in the XY plane, counterclockwise, without repeating the first point.
    fn contour(&self, tolerance: Length) -> Vec<Point2>;

    fn extrude(self, height: Length) -> Extrusion<Self> {
        Extrusion::new(self, height)
    }

    fn rotate_extrude(self, angle: Angle) -> RotateExtrusion<Self> {
        RotateExtrusion::new(self, angle)
    }

    fn twist_extrude(self, height: Length, twist: Angle) -> TwistExtrusion<Self> {
        TwistExtrusion::new(self, height, twist)
    }
}
