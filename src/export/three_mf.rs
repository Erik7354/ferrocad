use std::fs::File;
use std::io::{self, Seek, Write};
use std::path::Path;

use threemf::model::mesh::{Mesh as ThreeMfMesh, Triangle, Triangles, Vertex, Vertices};
use threemf::model::{Base, BaseMaterials, Item, Object as ThreeMfObject, Unit};

use crate::color::Color;
use crate::mesh::Mesh;
use crate::model::Model;

pub fn write_3mf(model: &Model, path: impl AsRef<Path>) -> io::Result<()> {
    let mut file = File::create(path)?;
    write_3mf_to(model, &mut file)
}

fn write_3mf_to(model: &Model, writer: &mut (impl Write + Seek)) -> io::Result<()> {
    threemf::write::write(writer, to_threemf_model(model)).map_err(io::Error::other)
}

fn to_threemf_model(model: &Model) -> threemf::model::Model {
    let mut document = threemf::model::Model::default();
    document.unit = Unit::Millimeter;

    let colors = unique_colors(model);
    let materials_id = if colors.is_empty() { None } else { Some(1) };
    let mut next_id = if materials_id.is_some() { 2 } else { 1 };

    if let Some(id) = materials_id {
        document.resources.basematerials = Some(vec![BaseMaterials {
            id,
            base: colors
                .iter()
                .map(|color| Base {
                    name: color.displaycolor(),
                    displaycolor: color.displaycolor(),
                })
                .collect(),
        }]);
    }

    for object in model.objects() {
        let id = next_id;
        next_id += 1;

        let (pid, pindex) = match object.color {
            Some(color) => {
                let index = colors
                    .iter()
                    .position(|c| *c == color)
                    .expect("color indexed");
                (materials_id, Some(index))
            }
            None => (None, None),
        };

        document.resources.object.push(ThreeMfObject {
            id,
            partnumber: None,
            name: Some(object.name.clone()),
            pid,
            pindex,
            mesh: Some(to_threemf_mesh(&object.mesh)),
            components: None,
        });

        document.build.item.push(Item {
            objectid: id,
            transform: None,
            partnumber: None,
        });
    }

    document
}

fn unique_colors(model: &Model) -> Vec<Color> {
    let mut colors = Vec::new();
    for object in model.objects() {
        if let Some(color) = object.color
            && !colors.contains(&color)
        {
            colors.push(color);
        }
    }
    colors
}

fn to_threemf_mesh(mesh: &Mesh) -> ThreeMfMesh {
    ThreeMfMesh {
        vertices: Vertices {
            vertex: mesh
                .vertices
                .iter()
                .map(|point| Vertex {
                    x: point.x,
                    y: point.y,
                    z: point.z,
                })
                .collect(),
        },
        triangles: Triangles {
            triangle: mesh
                .triangles
                .iter()
                .map(|&[v1, v2, v3]| Triangle { v1, v2, v3 })
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    use crate::ToLength;
    use crate::model::Object;
    use crate::solid::Solid;
    use crate::solid::cuboid::Cuboid;

    fn write_and_read(model: &Model) -> threemf::model::Model {
        let mut cursor = Cursor::new(Vec::new());
        write_3mf_to(model, &mut cursor).expect("3mf written");
        let bytes = cursor.into_inner();
        let mut models = threemf::read::read(Cursor::new(bytes)).expect("3mf parsed");
        assert_eq!(models.len(), 1);
        models.pop().unwrap()
    }

    #[test]
    fn write_3mf_writes_cuboid() {
        let mut model = Model::new();
        model.add(Object::new(
            "body",
            Cuboid::new(20.cm(), 30.cm(), 10.cm()).mesh(1.mm()),
        ));

        let parsed = write_and_read(&model);
        assert!(matches!(parsed.unit, Unit::Millimeter));

        let object = &parsed.resources.object[0];
        assert_eq!(object.name.as_deref(), Some("body"));
        let mesh = object.mesh.as_ref().expect("mesh");
        assert_eq!(mesh.vertices.vertex.len(), 8);
        assert_eq!(mesh.triangles.triangle.len(), 12);
        assert!(mesh.vertices.vertex.contains(&Vertex {
            x: 100.0,
            y: 150.0,
            z: 50.0,
        }));
        assert_eq!(parsed.build.item.len(), 1);
        assert_eq!(parsed.build.item[0].objectid, object.id);
    }

    fn colored_model() -> (Model, Color, Color) {
        let red = Color::rgb(200, 40, 40);
        let blue = Color::rgb(40, 40, 200);

        let mut model = Model::new();
        model.add(Object::new("body", Cuboid::cube(10.mm()).mesh(1.mm())).with_color(red));
        model.add(Object::new("lid", Cuboid::cube(8.mm()).mesh(1.mm())).with_color(blue));
        (model, red, blue)
    }

    #[test]
    fn write_3mf_assigns_base_materials() {
        let (model, red, blue) = colored_model();
        let document = to_threemf_model(&model);

        let materials = &document
            .resources
            .basematerials
            .as_ref()
            .expect("materials")[0];
        let body = document
            .resources
            .object
            .iter()
            .find(|object| object.name.as_deref() == Some("body"))
            .expect("body");
        let lid = document
            .resources
            .object
            .iter()
            .find(|object| object.name.as_deref() == Some("lid"))
            .expect("lid");

        assert_eq!(body.pid, Some(materials.id));
        assert_eq!(lid.pid, Some(materials.id));
        assert_eq!(
            materials.base[body.pindex.expect("body pindex")].displaycolor,
            red.displaycolor()
        );
        assert_eq!(
            materials.base[lid.pindex.expect("lid pindex")].displaycolor,
            blue.displaycolor()
        );
    }

    #[test]
    fn write_3mf_preserves_names_and_color_indices() {
        let (model, _, _) = colored_model();
        let parsed = write_and_read(&model);

        assert_eq!(parsed.resources.object.len(), 2);
        assert_eq!(parsed.build.item.len(), 2);

        let body = parsed
            .resources
            .object
            .iter()
            .find(|object| object.name.as_deref() == Some("body"))
            .expect("body");
        let lid = parsed
            .resources
            .object
            .iter()
            .find(|object| object.name.as_deref() == Some("lid"))
            .expect("lid");

        assert_eq!(body.pid, Some(1));
        assert_eq!(lid.pid, Some(1));
        assert_ne!(body.pindex, lid.pindex);
    }
}
