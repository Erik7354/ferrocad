use crate::{length::Length, mesh::Mesh};

pub mod cuboid;
pub mod cylinder;

pub use cuboid::Cuboid;
pub use cylinder::Cylinder;

/// A Solid represents a 3-dimensional volume.
pub trait Solid {
    fn mesh(&self, tolerance: Length) -> Mesh;
}
