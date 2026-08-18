use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use crate::mesh::{Mesh, Point3};

pub fn write_stl(mesh: &Mesh, path: impl AsRef<Path>) -> io::Result<()> {
    let mut file = File::create(path)?;
    write_stl_to(mesh, &mut file)
}

fn write_stl_to(mesh: &Mesh, writer: &mut impl Write) -> io::Result<()> {
    let triangles: Vec<stl_io::Triangle> = mesh
        .triangles
        .iter()
        .map(|triangle| {
            let a = mesh.vertices[triangle[0]];
            let b = mesh.vertices[triangle[1]];
            let c = mesh.vertices[triangle[2]];

            let normal = calculate_normal(a, b, c);

            stl_io::Triangle {
                normal: stl_io::Normal::new([normal[0] as f32, normal[1] as f32, normal[2] as f32]),
                vertices: [vertex(a), vertex(b), vertex(c)],
            }
        })
        .collect();

    stl_io::write_stl(writer, triangles.iter())
}

fn vertex(point: Point3) -> stl_io::Vertex {
    stl_io::Vertex::new([point.x as f32, point.y as f32, point.z as f32])
}

fn calculate_normal(a: Point3, b: Point3, c: Point3) -> [f64; 3] {
    let ab = [b.x - a.x, b.y - a.y, b.z - a.z];
    let ac = [c.x - a.x, c.y - a.y, c.z - a.z];

    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];

    let length = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();

    if length == 0.0 {
        return [0.0, 0.0, 0.0];
    }

    [cross[0] / length, cross[1] / length, cross[2] / length]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::solid::Solid;
    use crate::solid::cuboid::Cuboid;

    #[test]
    fn write_stl_writes_cuboid() {
        let cuboid = Cuboid::new(20.cm(), 30.cm(), 10.cm());
        let mesh = cuboid.mesh(1.mm());

        let mut bytes = Vec::new();
        write_stl_to(&mesh, &mut bytes).expect("stl written");

        let indexed = stl_io::read_stl(&mut std::io::Cursor::new(&bytes)).expect("stl parsed");
        assert_eq!(indexed.vertices.len(), 8);
        assert_eq!(indexed.faces.len(), 12);
        assert!(
            indexed
                .vertices
                .contains(&stl_io::Vertex::new([100.0, 150.0, 50.0]))
        );
    }
}
