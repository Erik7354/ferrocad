use crate::color::Color;
use crate::mesh::Mesh;

/// A printable assembly: named, optionally colored meshes.
#[derive(Debug, Clone, Default)]
pub struct Model {
    objects: Vec<Object>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &[Object] {
        &self.objects
    }
}

/// One object in a [`Model`]: a tessellated mesh with a name and optional color.
#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub mesh: Mesh,
    pub color: Option<Color>,
}

impl Object {
    pub fn new(name: impl Into<String>, mesh: Mesh) -> Self {
        Self {
            name: name.into(),
            mesh,
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::solid::Solid;
    use crate::solid::cuboid::Cuboid;

    #[test]
    fn new_object_keeps_name_and_mesh() {
        let mesh = Cuboid::new(20.cm(), 30.cm(), 10.cm()).mesh(1.mm());
        let object = Object::new("body", mesh);

        assert_eq!(object.name, "body");
        assert_eq!(object.mesh.vertices.len(), 8);
        assert_eq!(object.mesh.triangles.len(), 12);
        assert!(object.color.is_none());
    }

    #[test]
    fn add_preserves_name_and_color() {
        let mut model = Model::new();
        model.add(
            Object::new("body", Cuboid::cube(10.mm()).mesh(1.mm()))
                .with_color(Color::rgb(200, 40, 40)),
        );

        let object = &model.objects()[0];
        assert_eq!(object.name, "body");
        assert_eq!(object.color, Some(Color::rgb(200, 40, 40)));
    }
}
