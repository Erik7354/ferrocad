use crate::length::Length;
use crate::mesh::Mesh;

use super::{Solid, boolean};

/// Boolean union of two solids (A ∪ B).
///
/// The children tessellate during mesh generation. Manifold then combines the
/// meshes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Union<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> Union<A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Solid, B: Solid> Solid for Union<A, B> {
    fn mesh(&self, tolerance: Length) -> Mesh {
        boolean::union(&self.a.mesh(tolerance), &self.b.mesh(tolerance))
    }
}

/// Boolean difference of two solids (A \ B).
///
/// The first solid is the body. The second solid is the cut. The children
/// tessellate during mesh generation. Manifold then combines the meshes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Difference<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> Difference<A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Solid, B: Solid> Solid for Difference<A, B> {
    fn mesh(&self, tolerance: Length) -> Mesh {
        boolean::difference(&self.a.mesh(tolerance), &self.b.mesh(tolerance))
    }
}

/// Boolean intersection of two solids (A ∩ B).
///
/// The children tessellate during mesh generation. Manifold then combines the
/// meshes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersection<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> Intersection<A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Solid, B: Solid> Solid for Intersection<A, B> {
    fn mesh(&self, tolerance: Length) -> Mesh {
        boolean::intersection(&self.a.mesh(tolerance), &self.b.mesh(tolerance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::length::Length;
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

    #[test]
    fn cube_minus_inner_cube_has_shell_volume() {
        let mesh = Cuboid::cube(20.mm())
            .difference(Cuboid::cube(10.mm()))
            .mesh(1.mm());

        assert_volume_near(&mesh, 7000.0, 1e-3);
    }

    #[test]
    fn cube_intersection_inner_cube_keeps_the_smaller_volume() {
        let mesh = Cuboid::cube(20.mm())
            .intersection(Cuboid::cube(10.mm()))
            .mesh(1.mm());

        assert_volume_near(&mesh, 1000.0, 1e-3);
    }

    #[test]
    fn cube_minus_itself_is_empty() {
        let mesh = Cuboid::cube(10.mm())
            .difference(Cuboid::cube(10.mm()))
            .mesh(1.mm());

        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn small_cube_minus_large_cube_is_empty() {
        let mesh = Cuboid::cube(10.mm())
            .difference(Cuboid::cube(20.mm()))
            .mesh(1.mm());

        assert!(mesh.vertices.is_empty());
        assert!(mesh.triangles.is_empty());
    }

    #[test]
    fn plate_minus_through_hole_loses_the_cylinder_volume() {
        let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm());
        let hole = Cylinder::new(5.mm(), 10.mm());
        let mesh = plate.difference(hole).mesh(500.um());

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
        let mesh = left.union(right).mesh(1.mm());

        assert_volume_near(&mesh, 2000.0, 1e-3);
    }

    #[test]
    fn pose_then_difference_matches_difference_then_pose() {
        let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm());
        let hole = Cylinder::new(5.mm(), 10.mm());
        let offset = 8.mm();

        let posed_then_cut = plate
            .translate(offset, Length::ZERO, Length::ZERO)
            .difference(hole.translate(offset, Length::ZERO, Length::ZERO))
            .mesh(500.um());
        let cut_then_posed = plate
            .difference(hole)
            .translate(offset, Length::ZERO, Length::ZERO)
            .mesh(500.um());

        assert_volume_near(&posed_then_cut, cut_then_posed.signed_volume(), 1.0);

        let bbox = |mesh: &Mesh| {
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
        };
        let (min_a, max_a) = bbox(&posed_then_cut);
        let (min_b, max_b) = bbox(&cut_then_posed);
        for i in 0..3 {
            assert!((min_a[i] - min_b[i]).abs() < 0.6);
            assert!((max_a[i] - max_b[i]).abs() < 0.6);
        }
    }
}
