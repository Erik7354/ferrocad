use crate::color::Color;
use crate::mesh::Mesh;
use crate::model::Model;

use super::xml::{
    Base, BaseMaterials, Item, Mesh as ThreeMfMesh, Object as ThreeMfObject, Triangle, Triangles,
    Unit, Vertex, Vertices,
};
use super::xml::{Metadata, Model as XmlModel};

/// Convert a ferrocad model to a 3MF document.
pub fn to_document(model: &Model, metadata: Vec<Metadata>) -> XmlModel {
    let mut document = XmlModel {
        unit: Unit::Millimeter,
        metadata,
        ..XmlModel::default()
    };

    let colors = unique_colors(model);
    let materials_id = if colors.is_empty() { None } else { Some(1) };
    let start_id = if materials_id.is_some() { 2 } else { 1 };

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

    for (id, object) in (start_id..).zip(model.objects()) {
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
