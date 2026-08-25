mod document;
mod package;
mod xml;

use std::fs::File;
use std::io::{self, Seek, Write};
use std::path::Path;

use crate::model::Model;

use document::to_document;
use package::write_package;
use xml::Metadata;

/// Document metadata for a 3MF export.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThreeMfMetadata {
    pub title: Option<String>,
    pub designer: Option<String>,
    pub description: Option<String>,
}

/// Builder that writes a 3MF file.
///
/// A 3MF file is a ZIP archive. The archive uses the Open Packaging
/// Conventions (OPC). The package contains three files:
///
/// - `[Content_Types].xml` gives the MIME type of each file extension
/// - `_rels/.rels` points to the 3D model
/// - `3D/3dmodel.model` contains the model in XML
///
/// The model has resources (meshes and optional base materials) and a
/// build list. The build list places each object on the plate.
/// Title, designer and description become metadata in the model.
pub struct ThreeMfExport<'a> {
    model: &'a Model,
    metadata: ThreeMfMetadata,
}

impl ThreeMfMetadata {
    fn to_xml_entries(&self) -> Vec<Metadata> {
        let mut entries = Vec::new();
        if let Some(title) = &self.title {
            entries.push(Metadata {
                name: "Title".to_owned(),
                value: Some(title.clone()),
            });
        }
        if let Some(designer) = &self.designer {
            entries.push(Metadata {
                name: "Designer".to_owned(),
                value: Some(designer.clone()),
            });
        }
        if let Some(description) = &self.description {
            entries.push(Metadata {
                name: "Description".to_owned(),
                value: Some(description.clone()),
            });
        }
        entries
    }
}

impl<'a> ThreeMfExport<'a> {
    /// Start a 3MF export.
    pub fn new(model: &'a Model) -> Self {
        Self {
            model,
            metadata: ThreeMfMetadata::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = Some(title.into());
        self
    }

    pub fn with_designer(mut self, designer: impl Into<String>) -> Self {
        self.metadata.designer = Some(designer.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    pub fn with_metadata(mut self, metadata: ThreeMfMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Write the 3MF package to a file.
    pub fn write(self, path: impl AsRef<Path>) -> io::Result<()> {
        let mut file = File::create(path)?;
        self.write_to(&mut file)
    }

    fn write_to(self, writer: &mut (impl Write + Seek)) -> io::Result<()> {
        let document = to_document(self.model, self.metadata.to_xml_entries());
        write_package(writer, &document)
    }
}

/// Write a 3MF file.
pub fn write_3mf(model: &Model, path: impl AsRef<Path>) -> io::Result<()> {
    ThreeMfExport::new(model).write(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    use serde::Deserialize;
    use zip::ZipArchive;

    use super::xml::{Unit, Vertex};
    use crate::ToLength;
    use crate::color::Color;
    use crate::model::Object;
    use crate::solid::Solid;
    use crate::solid::cuboid::Cuboid;

    fn write_bytes(export: ThreeMfExport<'_>) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        export.write_to(&mut cursor).expect("3mf written");
        cursor.into_inner()
    }

    fn write_and_read(model: &Model) -> xml::Model {
        read_model(&write_bytes(ThreeMfExport::new(model)))
    }

    fn read_model(bytes: &[u8]) -> xml::Model {
        let mut zip = ZipArchive::new(Cursor::new(bytes)).expect("zip");
        let mut models = Vec::new();
        for index in 0..zip.len() {
            let file = zip.by_index(index).expect("entry");
            if file.name().ends_with(".model") {
                let mut de = quick_xml::de::Deserializer::from_reader(BufReader::new(file));
                models.push(xml::Model::deserialize(&mut de).expect("model parsed"));
            }
        }
        assert_eq!(models.len(), 1);
        models.pop().unwrap()
    }

    fn zip_names(bytes: &[u8]) -> Vec<String> {
        let mut zip = ZipArchive::new(Cursor::new(bytes)).expect("zip");
        let mut names: Vec<String> = (0..zip.len())
            .map(|index| zip.by_index(index).expect("entry").name().to_owned())
            .collect();
        names.sort();
        names
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
        let document = to_document(&model, Vec::new());

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

    #[test]
    fn zip_contains_core_entries() {
        let mut model = Model::new();
        model.add(Object::new("body", Cuboid::cube(10.mm()).mesh(1.mm())));
        let bytes = write_bytes(ThreeMfExport::new(&model));
        let names = zip_names(&bytes);
        assert_eq!(
            names,
            vec![
                "3D/3dmodel.model".to_owned(),
                "[Content_Types].xml".to_owned(),
                "_rels/.rels".to_owned(),
            ]
        );
        assert!(names.iter().all(|name| !name.starts_with("Metadata/")));
    }

    #[test]
    fn with_title_round_trips() {
        let mut model = Model::new();
        model.add(Object::new("body", Cuboid::cube(10.mm()).mesh(1.mm())));
        let parsed = read_model(&write_bytes(
            ThreeMfExport::new(&model).with_title("bracket"),
        ));
        let title = parsed
            .metadata
            .iter()
            .find(|entry| entry.name == "Title")
            .expect("Title");
        assert_eq!(title.value.as_deref(), Some("bracket"));
    }

    #[test]
    fn with_metadata_writes_designer() {
        let mut model = Model::new();
        model.add(Object::new("body", Cuboid::cube(10.mm()).mesh(1.mm())));
        let parsed = read_model(&write_bytes(ThreeMfExport::new(&model).with_metadata(
            ThreeMfMetadata {
                designer: Some("ferrocad".to_owned()),
                ..ThreeMfMetadata::default()
            },
        )));
        let designer = parsed
            .metadata
            .iter()
            .find(|entry| entry.name == "Designer")
            .expect("Designer");
        assert_eq!(designer.value.as_deref(), Some("ferrocad"));
    }
}
