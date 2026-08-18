use crate::{length::Length, mesh::Mesh};

pub mod cuboid;

/// A Solid represents a 3-dimensional volume.
pub trait Solid {
    fn mesh(&self, tolerance: Length) -> Mesh;
}
