use crate::angle::Angle;
use crate::length::Length;
use crate::sketch::{Circle, Polygon, Rectangle, Sketch};
use crate::solid::{
    Body, Cuboid, Cylinder, Difference, Extrusion, Intersection, RotateExtrusion, Sphere, Union,
};
use crate::transform::affine::{Affine2, Affine3};
use crate::transform::transformed::{TransformedSketch, TransformedSolid};

/// Affine pose operations in the XY plane.
///
/// Implement [`posed`](Self::posed) for a new sketch type. The other methods
/// compose into one [`TransformedSketch`]. If the value is already
/// transformed, [`posed`](Self::posed) returns that value and does not wrap
/// again.
pub trait Pose2: Sized {
    /// The origin-centered sketch inside the pose wrapper.
    type Inner;
    /// This sketch with its affine pose.
    ///
    /// If this value has no pose yet, the pose is identity.
    fn posed(self) -> TransformedSketch<Self::Inner>;
    fn translate(self, x: Length, y: Length) -> TransformedSketch<Self::Inner> {
        self.posed().then(Affine2::translate(x, y))
    }
    fn rotate(self, angle: Angle) -> TransformedSketch<Self::Inner> {
        self.posed().then(Affine2::rotate(angle))
    }
    fn scale(self, x: f64, y: f64) -> TransformedSketch<Self::Inner> {
        self.posed().then(Affine2::scale(x, y))
    }
    fn mirror(self, x: f64, y: f64) -> TransformedSketch<Self::Inner> {
        self.posed().then(Affine2::mirror(x, y))
    }
    fn multmatrix(self, matrix: Affine2) -> TransformedSketch<Self::Inner> {
        self.posed().then(matrix)
    }
}
impl Pose2 for Circle {
    type Inner = Self;
    fn posed(self) -> TransformedSketch<Self> {
        TransformedSketch::new(self)
    }
}
impl Pose2 for Rectangle {
    type Inner = Self;
    fn posed(self) -> TransformedSketch<Self> {
        TransformedSketch::new(self)
    }
}
impl Pose2 for Polygon {
    type Inner = Self;
    fn posed(self) -> TransformedSketch<Self> {
        TransformedSketch::new(self)
    }
}
impl<T> Pose2 for TransformedSketch<T> {
    type Inner = T;
    fn posed(self) -> TransformedSketch<T> {
        self
    }
}

/// Affine pose operations in 3D.
///
/// Implement [`posed`](Self::posed) for a new solid type. The other methods
/// compose into one [`TransformedSolid`]. If the value is already
/// transformed, [`posed`](Self::posed) returns that value and does not wrap
/// again.
pub trait Pose3: Sized {
    /// The origin-centered solid inside the pose wrapper.
    type Inner;
    /// This solid with its affine pose.
    ///
    /// If this value has no pose yet, the pose is identity.
    fn posed(self) -> TransformedSolid<Self::Inner>;
    fn translate(self, x: Length, y: Length, z: Length) -> TransformedSolid<Self::Inner> {
        self.posed().then(Affine3::translate(x, y, z))
    }
    fn rotate(self, x: Angle, y: Angle, z: Angle) -> TransformedSolid<Self::Inner> {
        self.posed().then(Affine3::rotate(x, y, z))
    }
    fn rotate_axis(self, angle: Angle, x: f64, y: f64, z: f64) -> TransformedSolid<Self::Inner> {
        self.posed().then(Affine3::rotate_axis(angle, x, y, z))
    }
    fn scale(self, x: f64, y: f64, z: f64) -> TransformedSolid<Self::Inner> {
        self.posed().then(Affine3::scale(x, y, z))
    }
    fn mirror(self, x: f64, y: f64, z: f64) -> TransformedSolid<Self::Inner> {
        self.posed().then(Affine3::mirror(x, y, z))
    }
    fn multmatrix(self, matrix: Affine3) -> TransformedSolid<Self::Inner> {
        self.posed().then(matrix)
    }
}
impl Pose3 for Cuboid {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl Pose3 for Cylinder {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl Pose3 for Sphere {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl<S: Sketch> Pose3 for Extrusion<S> {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl<S: Sketch> Pose3 for RotateExtrusion<S> {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl<A, B> Pose3 for Union<A, B> {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl<A, B> Pose3 for Difference<A, B> {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl<A, B> Pose3 for Intersection<A, B> {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
impl<T> Pose3 for TransformedSolid<T> {
    type Inner = T;
    fn posed(self) -> TransformedSolid<T> {
        self
    }
}

impl Body {
    pub fn translate(self, x: Length, y: Length, z: Length) -> Self {
        self.transform_by(Affine3::translate(x, y, z).to_manifold_transform())
    }

    pub fn rotate(self, x: Angle, y: Angle, z: Angle) -> Self {
        self.transform_by(Affine3::rotate(x, y, z).to_manifold_transform())
    }

    pub fn rotate_axis(self, angle: Angle, x: f64, y: f64, z: f64) -> Self {
        self.transform_by(Affine3::rotate_axis(angle, x, y, z).to_manifold_transform())
    }

    pub fn scale(self, x: f64, y: f64, z: f64) -> Self {
        self.transform_by(Affine3::scale(x, y, z).to_manifold_transform())
    }

    pub fn mirror(self, x: f64, y: f64, z: f64) -> Self {
        self.transform_by(Affine3::mirror(x, y, z).to_manifold_transform())
    }

    pub fn multmatrix(self, matrix: Affine3) -> Self {
        self.transform_by(matrix.to_manifold_transform())
    }
}

impl Pose3 for Body {
    type Inner = Self;
    fn posed(self) -> TransformedSolid<Self> {
        TransformedSolid::new(self)
    }
}
