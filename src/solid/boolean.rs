//! Mesh boolean operations via Manifold.
//!
//! This module converts ferrocad meshes to Manifold, runs the boolean, and
//! converts the result back. Manifold types stay private to this module.

use manifold_csg::manifold::Manifold;

use crate::mesh::{Mesh, Point3};

/// Owned Manifold wrapper. The public [`super::Body`] type stores this.
#[derive(Clone)]
pub(crate) struct Kernel {
    manifold: Manifold,
}

impl Kernel {
    pub(crate) fn empty() -> Self {
        Self {
            manifold: Manifold::empty(),
        }
    }

    pub(crate) fn from_mesh(mesh: &Mesh) -> Self {
        Self {
            manifold: to_manifold(mesh),
        }
    }

    pub(crate) fn to_mesh(&self) -> Mesh {
        from_manifold(&self.manifold)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.manifold.is_empty()
    }

    pub(crate) fn union(&self, other: &Self) -> Self {
        check(self.manifold.union(&other.manifold))
    }

    pub(crate) fn difference(&self, other: &Self) -> Self {
        check(self.manifold.difference(&other.manifold))
    }

    pub(crate) fn intersection(&self, other: &Self) -> Self {
        check(self.manifold.intersection(&other.manifold))
    }

    pub(crate) fn batch_union(parts: &[Self]) -> Self {
        if parts.is_empty() {
            return Self::empty();
        }
        if let [only] = parts {
            return only.clone();
        }
        let manifolds: Vec<Manifold> = parts.iter().map(|part| part.manifold.clone()).collect();
        check(Manifold::batch_union(&manifolds))
    }

    pub(crate) fn batch_difference(parts: &[Self]) -> Self {
        if parts.is_empty() {
            return Self::empty();
        }
        if let [only] = parts {
            return only.clone();
        }
        let manifolds: Vec<Manifold> = parts.iter().map(|part| part.manifold.clone()).collect();
        check(Manifold::batch_difference(&manifolds))
    }

    pub(crate) fn transform(&self, m: &[f64; 12]) -> Self {
        let result = self.manifold.transform(m);
        result
            .status()
            .unwrap_or_else(|err| panic!("manifold transform failed: {err}"));
        Self { manifold: result }
    }
}

fn check(manifold: Manifold) -> Kernel {
    manifold
        .status()
        .unwrap_or_else(|err| panic!("manifold boolean failed: {err}"));
    Kernel { manifold }
}

fn to_manifold(mesh: &Mesh) -> Manifold {
    if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
        return Manifold::empty();
    }

    let mut vert_props = Vec::with_capacity(mesh.vertices.len() * 3);
    for point in &mesh.vertices {
        vert_props.push(point.x);
        vert_props.push(point.y);
        vert_props.push(point.z);
    }

    let mut tri_indices = Vec::with_capacity(mesh.triangles.len() * 3);
    for triangle in &mesh.triangles {
        tri_indices.push(triangle[0] as u64);
        tri_indices.push(triangle[1] as u64);
        tri_indices.push(triangle[2] as u64);
    }

    Manifold::from_mesh_f64(&vert_props, 3, &tri_indices)
        .unwrap_or_else(|err| panic!("manifold ingest failed: {err}"))
}

fn from_manifold(manifold: &Manifold) -> Mesh {
    if manifold.is_empty() {
        return Mesh::new(Vec::new(), Vec::new());
    }

    let (vert_props, n_props, tri_indices) = manifold.to_mesh_f64();
    assert!(
        n_props >= 3,
        "manifold mesh must store x, y, and z for each vertex"
    );

    let vertex_count = vert_props.len() / n_props;
    let mut vertices = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let base = i * n_props;
        vertices.push(Point3::new(
            vert_props[base],
            vert_props[base + 1],
            vert_props[base + 2],
        ));
    }

    let mut triangles = Vec::with_capacity(tri_indices.len() / 3);
    for chunk in tri_indices.chunks_exact(3) {
        triangles.push([chunk[0] as usize, chunk[1] as usize, chunk[2] as usize]);
    }

    Mesh::new(vertices, triangles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::solid::Solid;
    use crate::solid::cuboid::Cuboid;
    use crate::solid::cylinder::Cylinder;
    use crate::solid::sphere::Sphere;
    use crate::transform::Pose3;

    #[test]
    fn cuboid_ingests() {
        Kernel::from_mesh(&Cuboid::cube(10.mm()).mesh(1.mm()));
    }

    #[test]
    fn cylinder_ingests() {
        Kernel::from_mesh(&Cylinder::new(20.mm(), 10.mm()).mesh(1.mm()));
    }

    #[test]
    fn sphere_ingests() {
        Kernel::from_mesh(&Sphere::new(10.mm()).mesh(1.mm()));
    }

    #[test]
    fn posed_cuboid_ingests() {
        Kernel::from_mesh(
            &Cuboid::cube(10.mm())
                .translate(5.mm(), -2.mm(), 3.mm())
                .mesh(1.mm()),
        );
    }

    #[test]
    fn empty_mesh_becomes_an_empty_manifold() {
        let kernel = Kernel::from_mesh(&Mesh::new(Vec::new(), Vec::new()));
        assert!(kernel.is_empty());
    }
}
