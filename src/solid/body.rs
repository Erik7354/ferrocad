use std::fmt;

use crate::length::Length;
use crate::mesh::Mesh;

use super::Solid;
use super::boolean::Kernel;

/// A tessellated solid that runs CSG immediately.
///
/// [`Solid::bake`] commits a chord tolerance by meshing the input and keeping
/// the Manifold. Later [`mesh`](Self::mesh) calls ignore their tolerance
/// argument and export that result. Use the typed
/// [`Union`](super::Union) / [`Difference`](super::Difference) /
/// [`Intersection`](super::Intersection) tree when you still need to remesh
/// primitives at another quality.
///
/// Inherent `union`, `difference`, and `intersection` return [`Body`], so a
/// loop accumulator keeps one type. The [`Solid`] trait methods still wrap
/// into the typed tree.
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
    use crate::solid::cuboid::Cuboid;
    use crate::solid::cylinder::Cylinder;
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
    fn baked_plate_hole_loop_matches_typed_difference_chain() {
        let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm());
        let holes = [
            Cylinder::new(10.mm(), 2.mm()).translate(-10.mm(), -8.mm(), Length::ZERO),
            Cylinder::new(10.mm(), 2.mm()).translate(10.mm(), -8.mm(), Length::ZERO),
            Cylinder::new(10.mm(), 2.mm()).translate(-10.mm(), 8.mm(), Length::ZERO),
        ];
        let typed = plate
            .difference(holes[0])
            .difference(holes[1])
            .difference(holes[2])
            .mesh(500.um());

        let mut body = plate.bake(500.um());
        for hole in holes {
            body = body.difference(hole);
        }

        assert_volume_near(&body.mesh(500.um()), typed.signed_volume(), 1.0);

        let batched = Body::difference_all(plate, holes, 500.um());
        assert_volume_near(&batched.mesh(500.um()), typed.signed_volume(), 1.0);
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
    fn bake_then_translate_matches_typed_pose() {
        let baked = Cuboid::cube(10.mm())
            .bake(1.mm())
            .translate(5.mm(), -2.mm(), 3.mm())
            .mesh(1.mm());
        let typed = Cuboid::cube(10.mm())
            .translate(5.mm(), -2.mm(), 3.mm())
            .mesh(1.mm());

        assert_volume_near(&baked, typed.signed_volume(), 1e-3);
        let (min_a, max_a) = bbox(&baked);
        let (min_b, max_b) = bbox(&typed);
        for i in 0..3 {
            assert!((min_a[i] - min_b[i]).abs() < 1e-6);
            assert!((max_a[i] - max_b[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn body_mesh_matches_typed_mesh_at_bake_tolerance() {
        let cube = Cuboid::cube(10.mm());
        let baked = cube.bake(1.mm()).mesh(99.mm());
        let typed = cube.mesh(1.mm());

        assert_volume_near(&baked, typed.signed_volume(), 1e-3);
        let (min_a, max_a) = bbox(&baked);
        let (min_b, max_b) = bbox(&typed);
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
}
