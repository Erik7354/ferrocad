use crate::angle::Angle;
use crate::length::Length;
use crate::mesh::Mesh;
use crate::sketch::Sketch;
use crate::solid::{Body, Solid};
use crate::transform::affine::{Affine2, Affine3};

/// A sketch with a 2D affine pose.
///
/// The inner sketch stays origin-centered. Further affine methods compose
/// into [`affine`](Self::affine) instead of wrapping again.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformedSketch<T> {
    pub shape: T,
    pub affine: Affine2,
}

impl<T> TransformedSketch<T> {
    pub const fn new(shape: T) -> Self {
        Self {
            shape,
            affine: Affine2::IDENTITY,
        }
    }

    pub const fn with_affine(shape: T, affine: Affine2) -> Self {
        Self { shape, affine }
    }

    /// Apply `next` after the current pose (`next * affine`).
    pub fn then(self, next: Affine2) -> Self {
        Self {
            shape: self.shape,
            affine: next * self.affine,
        }
    }

    pub fn translate(self, x: Length, y: Length) -> Self {
        self.then(Affine2::translate(x, y))
    }

    pub fn rotate(self, angle: Angle) -> Self {
        self.then(Affine2::rotate(angle))
    }

    pub fn scale(self, x: f64, y: f64) -> Self {
        self.then(Affine2::scale(x, y))
    }

    pub fn mirror(self, x: f64, y: f64) -> Self {
        self.then(Affine2::mirror(x, y))
    }

    pub fn multmatrix(self, matrix: Affine2) -> Self {
        self.then(matrix)
    }
}

impl<S: Sketch> Sketch for TransformedSketch<S> {
    fn contour(&self, tolerance: Length) -> Vec<crate::mesh::Point2> {
        let mut contour: Vec<_> = self
            .shape
            .contour(tolerance)
            .into_iter()
            .map(|point| self.affine * point)
            .collect();
        if self.affine.determinant() < 0.0 {
            contour.reverse();
        }
        contour
    }
}

/// A solid with a 3D affine pose.
///
/// The inner solid stays origin-centered. Further affine methods compose
/// into [`affine`](Self::affine) instead of wrapping again.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformedSolid<T> {
    pub shape: T,
    pub affine: Affine3,
}

impl<T> TransformedSolid<T> {
    pub const fn new(shape: T) -> Self {
        Self {
            shape,
            affine: Affine3::IDENTITY,
        }
    }

    pub const fn with_affine(shape: T, affine: Affine3) -> Self {
        Self { shape, affine }
    }

    /// Apply `next` after the current pose (`next * affine`).
    pub fn then(self, next: Affine3) -> Self {
        Self {
            shape: self.shape,
            affine: next * self.affine,
        }
    }

    pub fn translate(self, x: Length, y: Length, z: Length) -> Self {
        self.then(Affine3::translate(x, y, z))
    }

    pub fn rotate(self, x: Angle, y: Angle, z: Angle) -> Self {
        self.then(Affine3::rotate(x, y, z))
    }

    pub fn rotate_axis(self, angle: Angle, x: f64, y: f64, z: f64) -> Self {
        self.then(Affine3::rotate_axis(angle, x, y, z))
    }

    pub fn scale(self, x: f64, y: f64, z: f64) -> Self {
        self.then(Affine3::scale(x, y, z))
    }

    pub fn mirror(self, x: f64, y: f64, z: f64) -> Self {
        self.then(Affine3::mirror(x, y, z))
    }

    pub fn multmatrix(self, matrix: Affine3) -> Self {
        self.then(matrix)
    }
}

impl TransformedSolid<Body> {
    fn into_body(self) -> Body {
        self.shape.transform_by(self.affine.to_manifold_transform())
    }

    /// Combine this posed body with `other` and keep a [`Body`].
    pub fn union(self, other: impl Solid) -> Body {
        self.into_body().union(other)
    }

    /// Cut `other` from this posed body and keep a [`Body`].
    pub fn difference(self, other: impl Solid) -> Body {
        self.into_body().difference(other)
    }

    /// Keep the overlap of this posed body and `other` and keep a [`Body`].
    pub fn intersection(self, other: impl Solid) -> Body {
        self.into_body().intersection(other)
    }

    /// Convex hull of this posed body. Keep a [`Body`].
    pub fn hull(self) -> Body {
        self.into_body().hull()
    }

    /// Convex hull of this posed body and `other`. Keep a [`Body`].
    pub fn hull_with(self, other: impl Solid) -> Body {
        self.into_body().hull_with(other)
    }
}

impl<S: Solid> Solid for TransformedSolid<S> {
    fn mesh(&self, tolerance: Length) -> Mesh {
        let mut mesh = self.shape.mesh(tolerance);
        for vertex in &mut mesh.vertices {
            *vertex = self.affine * *vertex;
        }
        if self.affine.determinant() < 0.0 {
            for triangle in &mut mesh.triangles {
                triangle.swap(1, 2);
            }
        }
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToAngle;
    use crate::ToLength;
    use crate::mesh::{Point2, Point3};
    use crate::sketch::{Circle, Rectangle, Sketch};
    use crate::solid::{Cuboid, Solid};
    use crate::transform::{Pose2, Pose3};

    fn assert_point3(actual: Point3, expected: Point3) {
        assert!(
            (actual.x - expected.x).abs() < 1e-9
                && (actual.y - expected.y).abs() < 1e-9
                && (actual.z - expected.z).abs() < 1e-9,
            "{actual:?} != {expected:?}"
        );
    }

    #[test]
    fn translate_keeps_the_shape() {
        let cuboid = Cuboid::cube(10.mm());
        let translated = cuboid.translate(5.mm(), -2.mm(), 8.mm());

        assert_eq!(translated.shape, cuboid);
        assert_eq!(
            translated.affine,
            Affine3::translate(5.mm(), -2.mm(), 8.mm())
        );
    }

    #[test]
    fn chained_translates_compose_into_one_transform() {
        let transformed: TransformedSolid<Cuboid> = Cuboid::cube(10.mm())
            .translate(5.mm(), 0.mm(), 0.mm())
            .translate(0.mm(), 3.mm(), 2.mm());

        assert_eq!(transformed.shape, Cuboid::cube(10.mm()));
        assert_eq!(
            transformed.affine,
            Affine3::translate(0.mm(), 3.mm(), 2.mm()) * Affine3::translate(5.mm(), 0.mm(), 0.mm())
        );
    }

    #[test]
    fn rotate_after_translate_stays_one_transformed_cuboid() {
        let transformed: TransformedSolid<Cuboid> = Cuboid::cube(10.mm())
            .translate(10.mm(), 0.mm(), 0.mm())
            .rotate(Angle::ZERO, Angle::ZERO, 90.deg());

        assert_eq!(transformed.shape, Cuboid::cube(10.mm()));
        assert_point3(
            transformed.affine * Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 10.0, 0.0),
        );
    }

    #[test]
    fn translated_solid_shifts_vertices_and_keeps_triangles() {
        let cuboid = Cuboid::new(20.mm(), 10.mm(), 8.mm());
        let original = cuboid.mesh(1.mm());
        let mesh = cuboid.translate(5.mm(), -3.mm(), 2.mm()).mesh(1.mm());

        assert_eq!(mesh.triangles, original.triangles);
        assert_eq!(mesh.vertices.len(), original.vertices.len());
        for (translated, original) in mesh.vertices.iter().zip(&original.vertices) {
            assert_eq!(
                *translated,
                Point3::new(original.x + 5.0, original.y - 3.0, original.z + 2.0)
            );
        }
    }

    #[test]
    fn zero_translation_is_the_original_mesh() {
        let cuboid = Cuboid::cube(10.mm());
        let original = cuboid.mesh(1.mm());
        let mesh = cuboid
            .translate(Length::ZERO, Length::ZERO, Length::ZERO)
            .mesh(1.mm());

        assert_eq!(mesh.vertices, original.vertices);
        assert_eq!(mesh.triangles, original.triangles);
    }

    #[test]
    fn chained_translations_add() {
        let mesh = Cuboid::cube(10.mm())
            .translate(5.mm(), 0.mm(), 0.mm())
            .translate(0.mm(), 3.mm(), 2.mm())
            .mesh(1.mm());
        let expected = Cuboid::cube(10.mm())
            .translate(5.mm(), 3.mm(), 2.mm())
            .mesh(1.mm());

        assert_eq!(mesh.vertices, expected.vertices);
        assert_eq!(mesh.triangles, expected.triangles);
    }

    #[test]
    fn translated_sketch_shifts_the_contour() {
        let rectangle = Rectangle::new(20.mm(), 10.mm());
        let original = rectangle.contour(1.mm());
        let contour = rectangle.translate(4.mm(), -1.mm()).contour(1.mm());

        assert_eq!(contour.len(), original.len());
        for (translated, original) in contour.iter().zip(&original) {
            assert_eq!(*translated, Point2::new(original.x + 4.0, original.y - 1.0));
        }
    }

    #[test]
    fn extruding_a_translated_sketch_matches_translating_the_extrusion_in_xy() {
        let circle = Circle::new(10.mm());
        let from_sketch = circle
            .translate(7.mm(), -4.mm())
            .extrude(20.mm())
            .mesh(1.mm());
        let from_solid = circle
            .extrude(20.mm())
            .translate(7.mm(), -4.mm(), Length::ZERO)
            .mesh(1.mm());

        assert_eq!(from_sketch.vertices, from_solid.vertices);
        assert_eq!(from_sketch.triangles, from_solid.triangles);
    }

    #[test]
    fn mirror_reverses_triangle_winding() {
        let cuboid = Cuboid::cube(10.mm());
        let original = cuboid.mesh(1.mm());
        let mesh = cuboid.mirror(1.0, 0.0, 0.0).mesh(1.mm());

        assert_eq!(mesh.triangles.len(), original.triangles.len());
        for (mirrored, original) in mesh.triangles.iter().zip(&original.triangles) {
            assert_eq!(*mirrored, [original[0], original[2], original[1]]);
        }
    }

    #[test]
    fn mirrored_cuboid_triangles_still_face_outward() {
        let mesh = Cuboid::cube(10.mm()).mirror(1.0, 0.0, 0.0).mesh(1.mm());

        for &[a, b, c] in &mesh.triangles {
            let ab = (
                mesh.vertices[b].x - mesh.vertices[a].x,
                mesh.vertices[b].y - mesh.vertices[a].y,
                mesh.vertices[b].z - mesh.vertices[a].z,
            );
            let ac = (
                mesh.vertices[c].x - mesh.vertices[a].x,
                mesh.vertices[c].y - mesh.vertices[a].y,
                mesh.vertices[c].z - mesh.vertices[a].z,
            );
            let normal = (
                ab.1 * ac.2 - ab.2 * ac.1,
                ab.2 * ac.0 - ab.0 * ac.2,
                ab.0 * ac.1 - ab.1 * ac.0,
            );
            let centroid = (
                (mesh.vertices[a].x + mesh.vertices[b].x + mesh.vertices[c].x) / 3.0,
                (mesh.vertices[a].y + mesh.vertices[b].y + mesh.vertices[c].y) / 3.0,
                (mesh.vertices[a].z + mesh.vertices[b].z + mesh.vertices[c].z) / 3.0,
            );

            assert!(normal.0 * centroid.0 + normal.1 * centroid.1 + normal.2 * centroid.2 > 0.0);
        }
    }

    #[test]
    fn generic_pose_composes_into_one_wrapper() {
        fn bump<S: Pose3>(s: S) -> TransformedSolid<S::Inner> {
            s.translate(1.mm(), 0.mm(), 0.mm())
        }
        let cuboid = Cuboid::cube(10.mm());
        let twice = bump(bump(cuboid));
        assert_eq!(twice.shape, cuboid);
        assert_eq!(
            twice.affine,
            Affine3::translate(1.mm(), 0.mm(), 0.mm()) * Affine3::translate(1.mm(), 0.mm(), 0.mm())
        );
    }

    #[test]
    fn mirror_keeps_a_sketch_contour_counterclockwise() {
        let contour = Rectangle::new(20.mm(), 10.mm())
            .mirror(1.0, 0.0)
            .contour(1.mm());

        assert_eq!(
            contour,
            vec![
                Point2::new(10.0, 5.0),
                Point2::new(-10.0, 5.0),
                Point2::new(-10.0, -5.0),
                Point2::new(10.0, -5.0),
            ]
        );
    }

    #[test]
    fn negative_scale_reverses_triangle_winding() {
        let cuboid = Cuboid::cube(10.mm());
        let original = cuboid.mesh(1.mm());
        let mesh = cuboid.scale(-1.0, 1.0, 1.0).mesh(1.mm());

        assert_eq!(mesh.triangles.len(), original.triangles.len());
        for (scaled, original) in mesh.triangles.iter().zip(&original.triangles) {
            assert_eq!(*scaled, [original[0], original[2], original[1]]);
        }
    }
}
