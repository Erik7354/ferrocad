use std::fmt;

use crate::length::Length;
use crate::mesh::Mesh;

use super::Solid;
use super::boolean::Kernel;

/// A tessellated solid that runs CSG immediately.
///
/// [`Solid::bake`] commits a chord tolerance by meshing the input and keeping
/// the Manifold. Later [`mesh`](Self::mesh) calls ignore their tolerance
/// argument and export that result.
///
/// Boolean CSG (`union`, `difference`, `intersection`), convex hull
/// (`hull`, `hull_with`, `hull_all`), and batch helpers (`union_all`,
/// `difference_all`) return [`Body`], so a loop accumulator keeps one type.
#[derive(Clone)]
pub struct Body {
    kernel: Kernel,
    tolerance: Length,
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body")
            .field("empty", &self.kernel.is_empty())
            .field("tolerance_mm", &self.tolerance.as_mm_f64())
            .finish()
    }
}

impl Body {
    /// Empty solid. Union identity. Difference and intersection with an empty
    /// body follow Manifold's empty-solid rules.
    pub fn empty() -> Self {
        Self {
            kernel: Kernel::empty(),
            tolerance: Length::ZERO,
        }
    }

    pub(crate) fn from_solid(solid: impl Solid, tolerance: Length) -> Self {
        Self {
            kernel: Kernel::from_mesh(&solid.mesh(tolerance)),
            tolerance,
        }
    }

    pub(crate) fn transform_by(self, m: [f64; 12]) -> Self {
        Self {
            kernel: self.kernel.transform(&m),
            tolerance: self.tolerance,
        }
    }

    fn with_other(
        self,
        other: impl Solid,
        combine: impl FnOnce(&Kernel, &Kernel) -> Kernel,
    ) -> Self {
        let other = Self::from_solid(other, self.tolerance);
        let tolerance = if self.kernel.is_empty() {
            other.tolerance
        } else {
            self.tolerance
        };
        Self {
            kernel: combine(&self.kernel, &other.kernel),
            tolerance,
        }
    }

    /// Combine this solid with `other` (A ∪ B) and keep the Manifold.
    pub fn union(self, other: impl Solid) -> Self {
        self.with_other(other, Kernel::union)
    }

    /// Cut `other` from this solid (A \ B) and keep the Manifold.
    pub fn difference(self, other: impl Solid) -> Self {
        self.with_other(other, Kernel::difference)
    }

    /// Keep the overlap of this solid and `other` (A ∩ B) and keep the Manifold.
    pub fn intersection(self, other: impl Solid) -> Self {
        self.with_other(other, Kernel::intersection)
    }

    /// Union every solid in `solids`, tessellated at `tolerance`.
    ///
    /// An empty iterator yields [`Body::empty`].
    pub fn union_all<I>(solids: I, tolerance: Length) -> Self
    where
        I: IntoIterator,
        I::Item: Solid,
    {
        let parts: Vec<Kernel> = solids
            .into_iter()
            .map(|solid| Kernel::from_mesh(&solid.mesh(tolerance)))
            .collect();
        if parts.is_empty() {
            return Self {
                kernel: Kernel::empty(),
                tolerance,
            };
        }
        Self {
            kernel: Kernel::batch_union(&parts),
            tolerance,
        }
    }

    /// Subtract every cut from `body`, tessellated at `tolerance`.
    pub fn difference_all<B, I>(body: B, cuts: I, tolerance: Length) -> Self
    where
        B: Solid,
        I: IntoIterator,
        I::Item: Solid,
    {
        let mut parts = vec![Kernel::from_mesh(&body.mesh(tolerance))];
        parts.extend(
            cuts.into_iter()
                .map(|solid| Kernel::from_mesh(&solid.mesh(tolerance))),
        );
        Self {
            kernel: Kernel::batch_difference(&parts),
            tolerance,
        }
    }

    /// Convex hull of this solid. Keep the Manifold.
    pub fn hull(self) -> Self {
        Self {
            kernel: self.kernel.hull(),
            tolerance: self.tolerance,
        }
    }

    /// Convex hull of this solid and `other`. Keep the Manifold.
    pub fn hull_with(self, other: impl Solid) -> Self {
        self.with_other(other, |a, b| Kernel::batch_hull(&[a.clone(), b.clone()]))
    }

    /// Convex hull of every solid in `solids`, tessellated at `tolerance`.
    ///
    /// An empty iterator yields [`Body::empty`].
    pub fn hull_all<I>(solids: I, tolerance: Length) -> Self
    where
        I: IntoIterator,
        I::Item: Solid,
    {
        let parts: Vec<Kernel> = solids
            .into_iter()
            .map(|solid| Kernel::from_mesh(&solid.mesh(tolerance)))
            .collect();
        if parts.is_empty() {
            return Self {
                kernel: Kernel::empty(),
                tolerance,
            };
        }
        Self {
            kernel: Kernel::batch_hull(&parts),
            tolerance,
        }
    }
}

impl Solid for Body {
    /// Export the stored Manifold. `tolerance` is ignored; tessellation was
    /// chosen at [`Solid::bake`].
    fn mesh(&self, _tolerance: Length) -> Mesh {
        self.kernel.to_mesh()
    }

    fn bake(self, _tolerance: Length) -> Body {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::mesh::Mesh;
    use crate::sketch::{Polygon, Sketch};
    use crate::solid::cuboid::Cuboid;
    use crate::solid::cylinder::Cylinder;
    use crate::solid::sphere::Sphere;
    use crate::transform::Pose3;

    fn assert_volume_near(mesh: &Mesh, expected: f64, abs_tol: f64) {
        let volume = mesh.signed_volume();
        assert!(
            volume > 0.0,
            "CSG mesh winding must give a positive volume, got {volume}"
        );
        assert!(
            (volume - expected).abs() < abs_tol,
            "volume {volume} is not near {expected} (tol {abs_tol})"
        );
    }

    fn bbox(mesh: &Mesh) -> ([f64; 3], [f64; 3]) {
        let mut min = [f64::MAX; 3];
        let mut max = [f64::MIN; 3];
        for vertex in &mesh.vertices {
            min[0] = min[0].min(vertex.x);
            min[1] = min[1].min(vertex.y);
            min[2] = min[2].min(vertex.z);
            max[0] = max[0].max(vertex.x);
            max[1] = max[1].max(vertex.y);
            max[2] = max[2].max(vertex.z);
        }
        (min, max)
    }

    #[test]
    fn loop_of_translated_cubes_keeps_body_and_adds_volumes() {
        let mut part = Cuboid::cube(10.mm()).bake(1.mm());
        for i in 1i64..8 {
            part = part.union(Cuboid::cube(10.mm()).translate(
                (i * 20).mm(),
                Length::ZERO,
                Length::ZERO,
            ));
        }
        let _: Body = part.clone();
        assert_volume_near(&part.mesh(1.mm()), 8000.0, 1e-3);
    }

    #[test]
    fn plate_hole_loop_matches_difference_all() {
        let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm());
        let holes = [
            Cylinder::new(10.mm(), 2.mm()).translate(-10.mm(), -8.mm(), Length::ZERO),
            Cylinder::new(10.mm(), 2.mm()).translate(10.mm(), -8.mm(), Length::ZERO),
            Cylinder::new(10.mm(), 2.mm()).translate(-10.mm(), 8.mm(), Length::ZERO),
        ];

        let mut body = plate.bake(500.um());
        for hole in holes {
            body = body.difference(hole);
        }

        let batched = Body::difference_all(plate, holes, 500.um());
        assert_volume_near(
            &body.mesh(500.um()),
            batched.mesh(500.um()).signed_volume(),
            1.0,
        );
    }

    #[test]
    fn union_all_matches_a_left_fold_of_body_union() {
        let cubes: Vec<_> = (0i64..4)
            .map(|i| Cuboid::cube(10.mm()).translate((i * 20).mm(), Length::ZERO, Length::ZERO))
            .collect();
        let batched = Body::union_all(cubes.clone(), 1.mm());
        let folded = cubes
            .into_iter()
            .map(|cube| cube.bake(1.mm()))
            .reduce(|a, b| a.union(b))
            .unwrap();

        assert_volume_near(
            &batched.mesh(1.mm()),
            folded.mesh(1.mm()).signed_volume(),
            1e-3,
        );
    }

    #[test]
    fn empty_union_cube_matches_baked_cube() {
        let cube = Cuboid::cube(10.mm());
        let baked = cube.bake(1.mm());
        let from_empty = Body::empty().union(baked.clone());

        assert_volume_near(
            &from_empty.mesh(1.mm()),
            baked.mesh(1.mm()).signed_volume(),
            1e-3,
        );
        assert_volume_near(
            &from_empty.mesh(1.mm()),
            cube.mesh(1.mm()).signed_volume(),
            1e-3,
        );
    }

    #[test]
    fn bake_then_translate_matches_posed_primitive_mesh() {
        let baked = Cuboid::cube(10.mm())
            .bake(1.mm())
            .translate(5.mm(), -2.mm(), 3.mm())
            .mesh(1.mm());
        let posed = Cuboid::cube(10.mm())
            .translate(5.mm(), -2.mm(), 3.mm())
            .mesh(1.mm());

        assert_volume_near(&baked, posed.signed_volume(), 1e-3);
        let (min_a, max_a) = bbox(&baked);
        let (min_b, max_b) = bbox(&posed);
        for i in 0..3 {
            assert!((min_a[i] - min_b[i]).abs() < 1e-6);
            assert!((max_a[i] - max_b[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn body_mesh_matches_primitive_mesh_at_bake_tolerance() {
        let cube = Cuboid::cube(10.mm());
        let baked = cube.bake(1.mm()).mesh(99.mm());
        let primitive = cube.mesh(1.mm());

        assert_volume_near(&baked, primitive.signed_volume(), 1e-3);
        let (min_a, max_a) = bbox(&baked);
        let (min_b, max_b) = bbox(&primitive);
        for i in 0..3 {
            assert!((min_a[i] - min_b[i]).abs() < 1e-6);
            assert!((max_a[i] - max_b[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn posed_body_union_returns_body() {
        let moved = Pose3::translate(
            Cuboid::cube(10.mm()).bake(1.mm()),
            20.mm(),
            Length::ZERO,
            Length::ZERO,
        );
        let combined = moved.union(Cuboid::cube(10.mm()));
        let _: Body = combined.clone();
        assert_volume_near(&combined.mesh(1.mm()), 2000.0, 1e-3);
    }

    #[test]
    fn cube_minus_inner_cube_has_shell_volume() {
        let mesh = Cuboid::cube(20.mm())
            .bake(1.mm())
            .difference(Cuboid::cube(10.mm()))
            .mesh(1.mm());

        assert_volume_near(&mesh, 7000.0, 1e-3);
    }

    #[test]
    fn cube_intersection_inner_cube_keeps_the_smaller_volume() {
        let mesh = Cuboid::cube(20.mm())
            .bake(1.mm())
            .intersection(Cuboid::cube(10.mm()))
            .mesh(1.mm());

        assert_volume_near(&mesh, 1000.0, 1e-3);
    }

    #[test]
    fn cube_minus_itself_is_empty() {
        let mesh = Cuboid::cube(10.mm())
            .bake(1.mm())
            .difference(Cuboid::cube(10.mm()))
            .mesh(1.mm());

        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn small_cube_minus_large_cube_is_empty() {
        let mesh = Cuboid::cube(10.mm())
            .bake(1.mm())
            .difference(Cuboid::cube(20.mm()))
            .mesh(1.mm());

        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn plate_minus_through_hole_loses_the_cylinder_volume() {
        let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm());
        let hole = Cylinder::new(10.mm(), 5.mm());
        let mesh = plate.bake(500.um()).difference(hole).mesh(500.um());

        let plate_volume = plate.mesh(500.um()).signed_volume();
        let volume = mesh.signed_volume();
        let hole_volume = std::f64::consts::PI * 25.0 * 4.0;

        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.triangles.is_empty());
        assert!(volume > 0.0);
        assert!(volume < plate_volume);
        assert!(volume > plate_volume - hole_volume - 1.0);
        assert!(volume < plate_volume - hole_volume * 0.8);

        let min_xy = mesh
            .vertices
            .iter()
            .map(|vertex| (vertex.x * vertex.x + vertex.y * vertex.y).sqrt())
            .fold(f64::MAX, f64::min);
        assert!(
            (min_xy - 5.0).abs() < 0.5,
            "through-hole rim must stay near radius 5, got {min_xy}"
        );
    }

    #[test]
    fn disjoint_union_adds_volumes() {
        let left = Cuboid::cube(10.mm());
        let right = Cuboid::cube(10.mm()).translate(20.mm(), Length::ZERO, Length::ZERO);
        let mesh = left.bake(1.mm()).union(right).mesh(1.mm());

        assert_volume_near(&mesh, 2000.0, 1e-3);
    }

    #[test]
    fn pose_then_difference_matches_difference_then_pose() {
        let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm());
        let hole = Cylinder::new(10.mm(), 5.mm());
        let offset = 8.mm();
        let tol = 500.um();

        let posed_then_cut = plate
            .translate(offset, Length::ZERO, Length::ZERO)
            .bake(tol)
            .difference(hole.translate(offset, Length::ZERO, Length::ZERO))
            .mesh(tol);
        let cut_then_posed = plate
            .bake(tol)
            .difference(hole)
            .translate(offset, Length::ZERO, Length::ZERO)
            .mesh(tol);

        assert_volume_near(&posed_then_cut, cut_then_posed.signed_volume(), 1.0);

        let (min_a, max_a) = bbox(&posed_then_cut);
        let (min_b, max_b) = bbox(&cut_then_posed);
        for i in 0..3 {
            assert!((min_a[i] - min_b[i]).abs() < 0.6);
            assert!((max_a[i] - max_b[i]).abs() < 0.6);
        }
    }

    fn c_polygon() -> Polygon {
        Polygon::new(vec![
            [0.mm(), 0.mm()],
            [10.mm(), 0.mm()],
            [10.mm(), 2.mm()],
            [2.mm(), 2.mm()],
            [2.mm(), 8.mm()],
            [10.mm(), 8.mm()],
            [10.mm(), 10.mm()],
            [0.mm(), 10.mm()],
        ])
    }

    #[test]
    fn hull_of_a_cube_matches_the_cube() {
        let cube = Cuboid::cube(10.mm());
        let hulled = cube.bake(1.mm()).hull().mesh(1.mm());

        assert_volume_near(&hulled, 1000.0, 1e-3);
        let (min, max) = bbox(&hulled);
        for i in 0..3 {
            assert!((min[i] + 5.0).abs() < 1e-6);
            assert!((max[i] - 5.0).abs() < 1e-6);
        }
    }

    #[test]
    fn hull_of_a_c_prism_is_the_bounding_box() {
        let mesh = c_polygon().extrude(8.mm()).bake(1.mm()).hull().mesh(1.mm());

        assert_volume_near(&mesh, 800.0, 1e-3);
    }

    #[test]
    fn hull_of_two_disjoint_cubes_is_the_spanning_box() {
        let mesh = Cuboid::cube(10.mm())
            .bake(1.mm())
            .hull_with(Cuboid::cube(10.mm()).translate(20.mm(), Length::ZERO, Length::ZERO))
            .mesh(1.mm());

        assert_volume_near(&mesh, 3000.0, 1e-3);
    }

    #[test]
    fn hull_of_two_spheres_is_between_union_and_bbox() {
        let a = Sphere::new(10.mm());
        let b = Sphere::new(10.mm()).translate(40.mm(), Length::ZERO, Length::ZERO);
        let mesh = Body::hull_all([a.posed(), b], 500.um()).mesh(500.um());
        let union = a.bake(500.um()).union(b).mesh(500.um()).signed_volume();

        let volume = mesh.signed_volume();
        assert!(volume > 0.0);
        assert!(volume > union);
        assert!(volume < 60.0 * 20.0 * 20.0);

        let (min, max) = bbox(&mesh);
        assert!((min[0] + 10.0).abs() < 0.6);
        assert!((max[0] - 50.0).abs() < 0.6);
        assert!((min[1] + 10.0).abs() < 0.6);
        assert!((max[1] - 10.0).abs() < 0.6);
        assert!((min[2] + 10.0).abs() < 0.6);
        assert!((max[2] - 10.0).abs() < 0.6);
    }

    #[test]
    fn hull_all_of_one_solid_matches_unary_hull() {
        let prism = c_polygon().extrude(8.mm());
        let unary = prism.clone().bake(1.mm()).hull().mesh(1.mm());
        let batched = Body::hull_all([prism], 1.mm()).mesh(1.mm());

        assert_volume_near(&batched, unary.signed_volume(), 1e-3);
    }

    #[test]
    fn empty_hull_is_empty() {
        let empty_all = Body::hull_all(Vec::<Cuboid>::new(), 1.mm()).mesh(1.mm());
        assert!(empty_all.vertices.is_empty());
        assert!(empty_all.triangles.is_empty());

        let empty_unary = Body::empty().hull().mesh(1.mm());
        assert!(empty_unary.vertices.is_empty());
        assert!(empty_unary.triangles.is_empty());
    }

    #[test]
    fn pose_then_hull_matches_hull_then_pose() {
        let prism = c_polygon().extrude(8.mm());
        let offset = 8.mm();
        let tol = 1.mm();

        let posed_then_hull = prism
            .clone()
            .translate(offset, Length::ZERO, Length::ZERO)
            .bake(tol)
            .hull()
            .mesh(tol);
        let hull_then_posed = prism
            .bake(tol)
            .hull()
            .translate(offset, Length::ZERO, Length::ZERO)
            .mesh(tol);

        assert_volume_near(&posed_then_hull, hull_then_posed.signed_volume(), 1.0);

        let (min_a, max_a) = bbox(&posed_then_hull);
        let (min_b, max_b) = bbox(&hull_then_posed);
        for i in 0..3 {
            assert!((min_a[i] - min_b[i]).abs() < 0.6);
            assert!((max_a[i] - max_b[i]).abs() < 0.6);
        }
    }

    #[test]
    fn posed_body_hull_returns_body() {
        let moved = Pose3::translate(
            Cuboid::cube(10.mm()).bake(1.mm()),
            20.mm(),
            Length::ZERO,
            Length::ZERO,
        );
        let combined = moved.hull_with(Cuboid::cube(10.mm()));
        let _: Body = combined.clone();
        assert_volume_near(&combined.mesh(1.mm()), 3000.0, 1e-3);
    }
}
