use crate::{
    length::Length,
    mesh::{Mesh, Point3},
    solid::{Solid, chord_segments, circle_segments},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sphere {
    pub radius: Length,
}

impl Sphere {
    pub const fn new(radius: Length) -> Self {
        Self { radius }
    }

    pub const fn diameter(diameter: Length) -> Self {
        Self::new(Length::um(diameter.as_um() / 2))
    }
}

impl Solid for Sphere {
    fn mesh(&self, tolerance: Length) -> Mesh {
        let radius = self.radius.as_mm_f64();
        let tolerance = tolerance.as_mm_f64();
        let slices = circle_segments(radius, tolerance);
        let stacks = sphere_stacks(radius, tolerance);
        let rings = stacks - 1;

        let mut vertices = Vec::with_capacity(2 + rings * slices);
        vertices.push(Point3::new(0.0, 0.0, -radius));
        for ring in 0..rings {
            let phi = std::f64::consts::PI * (stacks - 1 - ring) as f64 / stacks as f64;
            for slice in 0..slices {
                let theta = 2.0 * std::f64::consts::PI * slice as f64 / slices as f64;
                vertices.push(sphere_point(radius, phi, theta));
            }
        }
        vertices.push(Point3::new(0.0, 0.0, radius));

        let north_pole = vertices.len() - 1;
        let mut triangles = Vec::with_capacity(2 * slices * rings);
        for slice in 0..slices {
            let next = (slice + 1) % slices;
            triangles.push([0, 1 + next, 1 + slice]);
            let north_ring = 1 + (rings - 1) * slices;
            triangles.push([north_pole, north_ring + slice, north_ring + next]);
        }
        for ring in 0..rings - 1 {
            let south = 1 + ring * slices;
            let north = south + slices;
            for slice in 0..slices {
                let next = (slice + 1) % slices;
                triangles.push([south + slice, south + next, north + next]);
                triangles.push([south + slice, north + next, north + slice]);
            }
        }

        Mesh::new(vertices, triangles)
    }
}

fn sphere_stacks(radius_mm: f64, tolerance_mm: f64) -> usize {
    chord_segments(
        std::f64::consts::PI * radius_mm.max(0.0),
        tolerance_mm,
        2,
        180,
    )
}

fn sphere_point(radius: f64, phi: f64, theta: f64) -> Point3 {
    Point3::new(
        radius * phi.sin() * theta.cos(),
        radius * phi.sin() * theta.sin(),
        radius * phi.cos(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;

    #[test]
    fn diameter_is_twice_the_radius() {
        assert_eq!(Sphere::diameter(20.mm()), Sphere::new(10.mm()));
        assert_eq!(Sphere::diameter(5.mm()), Sphere::new(2_500.um()));
    }

    #[test]
    fn mesh_segment_count_follows_tolerance() {
        let slices = circle_segments(10.0, 1.0);
        let stacks = sphere_stacks(10.0, 1.0);
        let mesh = Sphere::new(10.mm()).mesh(1.mm());

        assert_eq!(slices, 63);
        assert_eq!(stacks, 32);
        assert_eq!(mesh.vertices.len(), 2 + (stacks - 1) * slices);
        assert_eq!(mesh.triangles.len(), 2 * slices * (stacks - 1));
    }

    #[test]
    fn coarser_tolerance_uses_fewer_segments() {
        let fine = Sphere::new(10.mm()).mesh(1.mm());
        let coarse = Sphere::new(10.mm()).mesh(5.mm());

        assert!(coarse.vertices.len() < fine.vertices.len());
        assert!(coarse.triangles.len() < fine.triangles.len());
    }

    #[test]
    fn vertices_lie_on_the_sphere() {
        let mesh = Sphere::new(10.mm()).mesh(1.mm());

        assert_eq!(mesh.vertices[0], Point3::new(0.0, 0.0, -10.0));
        assert_eq!(
            mesh.vertices[mesh.vertices.len() - 1],
            Point3::new(0.0, 0.0, 10.0)
        );

        for vertex in &mesh.vertices {
            let radius = (vertex.x * vertex.x + vertex.y * vertex.y + vertex.z * vertex.z).sqrt();
            assert!((radius - 10.0).abs() < 1e-9);
        }
    }

    #[test]
    fn triangles_face_outward() {
        let mesh = Sphere::new(10.mm()).mesh(1.mm());

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
}
