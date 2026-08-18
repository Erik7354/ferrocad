#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Polygonal model: unique 3D points plus the triangles that connect them.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// Unique 3D points (geometry).
    pub vertices: Vec<Point3>,
    /// Faces as indices into `vertices` (topology); each triple is one triangle.
    pub triangles: Vec<[usize; 3]>,
}

impl Mesh {
    pub fn new(vertices: Vec<Point3>, triangles: Vec<[usize; 3]>) -> Self {
        Self {
            vertices,
            triangles,
        }
    }
}
