//! Affine transformations.
//!
//! Primitives (`Cuboid`, `Circle`, …) describe **shape** and stay centered on
//! the origin. Pose lives in a separate wrapper:
//!
//! - [`TransformedSketch`] holds a 2D [`Affine2`] around a [`Sketch`]
//! - [`Transformed`] holds a 3D [`Affine3`] around a [`Solid`]
//!
//! `translate`, `rotate`, `scale`, `mirror`, and `multmatrix` are the affine
//! operations. They all compose into that one matrix. Chaining them does not
//! nest wrappers:
//!
//! ```
//! use ferrocad::{Angle, Cuboid, Solid, ToAngle, ToLength, Transformed};
//!
//! let moved: Transformed<Cuboid> = Cuboid::cube(10.mm())
//!     .translate(10.mm(), 0.mm(), 0.mm())
//!     .rotate(Angle::ZERO, Angle::ZERO, 90.deg());
//! ```
//!
//! Each method applies **after** those already on the value, matching a method
//! chain.
//!
//! Orientation-reversing maps (a mirror, or a negative scale) flip triangle
//! winding / contour order so meshes stay outward-facing and sketches stay
//! counterclockwise.

mod affine;
mod transformed;

pub use affine::{Affine2, Affine3};
pub use transformed::{TransformedSketch, TransformedSolid};
