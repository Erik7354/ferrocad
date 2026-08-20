//! Affine transformations.
//!
//! Primitives (`Cuboid`, `Circle`, …) describe **shape** and stay centered on
//! the origin. Pose lives in a separate wrapper:
//!
//! - [`TransformedSketch`] holds a 2D [`Affine2`] around a [`Sketch`]
//! - [`TransformedSolid`] holds a 3D [`Affine3`] around a [`Solid`]
//!
//! [`Pose2`] and [`Pose3`] supply `translate`, `rotate`, `scale`, `mirror`,
//! and `multmatrix`. They all compose into that one matrix. Chaining them
//! does not nest wrappers, including through generic functions:
//!
//! ```
//! use ferrocad::{Angle, Cuboid, Pose3, ToAngle, ToLength, TransformedSolid};
//!
//! let moved: TransformedSolid<Cuboid> = Cuboid::cube(10.mm())
//!     .translate(10.mm(), 0.mm(), 0.mm())
//!     .rotate(Angle::ZERO, Angle::ZERO, 90.deg());
//! ```
//!
//! Each method applies **after** those already on the value, matching a method
//! chain. `A * B` on [`Affine3`] means apply `B` first, then `A`.
//!
//! Orientation-reversing maps (a mirror, or a negative scale) flip triangle
//! winding / contour order so meshes stay outward-facing and sketches stay
//! counterclockwise.
//!
//! # Not affine
//!
//! These change topology or measure geometry; they do not belong in the matrix:
//!
//! - `offset`, `hull`, `minkowski`
//! - boolean CSG (`union`, `difference`, `intersection`)
//! - `resize` (needs a bounding box, then can emit a scale)
//! - `color` (appearance, attached on [`crate::Object`])
//!
//! [`Sketch`]: crate::Sketch
//! [`Solid`]: crate::Solid

mod affine;
mod pose;
mod transformed;

pub use affine::{Affine2, Affine3};
pub use pose::{Pose2, Pose3};
pub use transformed::{TransformedSketch, TransformedSolid};
