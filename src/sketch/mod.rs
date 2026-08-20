use crate::length::Length;
use crate::mesh::Point2;
use crate::solid::Extrusion;

pub mod circle;
pub mod rectangle;

pub use circle::Circle;
pub use rectangle::Rectangle;

/// A Sketch is a flat shape in the XY plane.
pub trait Sketch: Sized {
    /// Closed contour in the XY plane, counterclockwise, without repeating the first point.
    fn contour(&self, tolerance: Length) -> Vec<Point2>;

    fn extrude(self, height: Length) -> Extrusion<Self> {
        Extrusion::new(self, height)
    }
}
