use crate::angle::Angle;
use crate::length::Length;
use crate::mesh::Mesh;
use crate::transform::{Affine3, TransformedSolid};

pub mod cuboid;
pub mod cylinder;
pub mod extrusion;
pub mod sphere;

pub use cuboid::Cuboid;
pub use cylinder::Cylinder;
pub use extrusion::Extrusion;
pub use sphere::Sphere;

/// A Solid represents a 3-dimensional volume.
///
/// Affine pose methods wrap the solid in [`Transformed`]; see [`crate::transform`].
pub trait Solid {
    fn mesh(&self, tolerance: Length) -> Mesh;

    fn translate(self, x: Length, y: Length, z: Length) -> TransformedSolid<Self>
    where
        Self: Sized,
    {
        TransformedSolid::new(self).translate(x, y, z)
    }

    fn rotate(self, x: Angle, y: Angle, z: Angle) -> TransformedSolid<Self>
    where
        Self: Sized,
    {
        TransformedSolid::new(self).rotate(x, y, z)
    }

    fn rotate_axis(self, angle: Angle, x: f64, y: f64, z: f64) -> TransformedSolid<Self>
    where
        Self: Sized,
    {
        TransformedSolid::new(self).rotate_axis(angle, x, y, z)
    }

    fn scale(self, x: f64, y: f64, z: f64) -> TransformedSolid<Self>
    where
        Self: Sized,
    {
        TransformedSolid::new(self).scale(x, y, z)
    }

    fn mirror(self, x: f64, y: f64, z: f64) -> TransformedSolid<Self>
    where
        Self: Sized,
    {
        TransformedSolid::new(self).mirror(x, y, z)
    }

    fn multmatrix(self, matrix: Affine3) -> TransformedSolid<Self>
    where
        Self: Sized,
    {
        TransformedSolid::new(self).multmatrix(matrix)
    }
}

/// Number of segments along `length_mm` so each chord is at most `tolerance_mm`.
pub(crate) fn chord_segments(
    length_mm: f64,
    tolerance_mm: f64,
    min_segments: usize,
    max_segments: usize,
) -> usize {
    if length_mm <= 0.0 {
        return min_segments;
    }

    let spacing = if tolerance_mm <= 0.0 {
        length_mm / max_segments as f64
    } else {
        tolerance_mm
    };

    ((length_mm / spacing).ceil() as usize).clamp(min_segments, max_segments)
}

/// Number of rim segments so each chord is at most `tolerance_mm` long.
pub(crate) fn circle_segments(radius_mm: f64, tolerance_mm: f64) -> usize {
    chord_segments(
        2.0 * std::f64::consts::PI * radius_mm.max(0.0),
        tolerance_mm,
        3,
        360,
    )
}
