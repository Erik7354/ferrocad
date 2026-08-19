use crate::{length::Length, mesh::Mesh};

pub mod cuboid;

pub use cuboid::Cuboid;

/// A Solid represents a 3-dimensional volume.
pub trait Solid {
    fn mesh(&self, tolerance: Length) -> Mesh;
}
