use crate::angle::Angle;
use crate::length::Length;
use crate::mesh::Point2;
use crate::solid::Extrusion;
use crate::transform::{Affine2, TransformedSketch};

pub mod circle;
pub mod rectangle;

pub use circle::Circle;
pub use rectangle::Rectangle;

/// A Sketch is a flat shape in the XY plane.
///
/// Affine pose methods wrap the sketch in [`TransformedSketch`]; see [`crate::transform`].
pub trait Sketch: Sized {
    /// Closed contour in the XY plane, counterclockwise, without repeating the first point.
    fn contour(&self, tolerance: Length) -> Vec<Point2>;

    fn extrude(self, height: Length) -> Extrusion<Self> {
        Extrusion::new(self, height)
    }

    fn translate(self, x: Length, y: Length) -> TransformedSketch<Self> {
        TransformedSketch::new(self).translate(x, y)
    }

    fn rotate(self, angle: Angle) -> TransformedSketch<Self> {
        TransformedSketch::new(self).rotate(angle)
    }

    fn scale(self, x: f64, y: f64) -> TransformedSketch<Self> {
        TransformedSketch::new(self).scale(x, y)
    }

    fn mirror(self, x: f64, y: f64) -> TransformedSketch<Self> {
        TransformedSketch::new(self).mirror(x, y)
    }

    fn multmatrix(self, matrix: Affine2) -> TransformedSketch<Self> {
        TransformedSketch::new(self).multmatrix(matrix)
    }
}
