use crate::{length::Length, mesh::Mesh};

pub mod cuboid;
pub mod cylinder;
pub mod sphere;

pub use cuboid::Cuboid;
pub use cylinder::Cylinder;
pub use sphere::Sphere;

/// A Solid represents a 3-dimensional volume.
pub trait Solid {
    fn mesh(&self, tolerance: Length) -> Mesh;
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
