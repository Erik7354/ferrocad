use std::io::{self, Seek, Write};

use quick_xml::{
    Writer,
    events::{BytesDecl, Event},
    se::Serializer,
};
use serde::Serialize;
use zip::{ZipWriter, write::SimpleFileOptions};

use super::xml::Model;

pub const MODEL_PATH: &str = "3D/3dmodel.model";

/// Write a 3MF ZIP package with the model file.
pub fn write_package(writer: impl Write + Seek, model: &Model) -> io::Result<()> {
    let mut archive = ZipWriter::new(writer);
    let options = SimpleFileOptions::default();

    archive.start_file("[Content_Types].xml", options)?;
    archive.write_all(content_types().as_bytes())?;

    archive.start_file("_rels/.rels", options)?;
    archive.write_all(relationships(MODEL_PATH).as_bytes())?;

    archive.start_file(MODEL_PATH, options)?;
    archive.write_all(&serialize_model(model)?)?;

    archive.finish()?;
    Ok(())
}

fn serialize_model(model: &Model) -> io::Result<Vec<u8>> {
    let mut xml = String::new();
    let mut ser = Serializer::with_root(&mut xml, Some("model")).map_err(io::Error::other)?;
    ser.indent(' ', 2);
    model.serialize(ser).map_err(io::Error::other)?;

    let mut out = Vec::new();
    let mut xml_writer = Writer::new_with_indent(&mut out, b' ', 2);
    xml_writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(io::Error::other)?;
    xml_writer.write_indent().map_err(io::Error::other)?;
    xml_writer.into_inner().write_all(xml.as_bytes())?;
    Ok(out)
}

fn content_types() -> String {
    String::from(
        "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">
	<Default
        Extension=\"model\"
        ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\" />
	<Default
        Extension=\"rels\"
        ContentType=\"application/vnd.openxmlformats-package.relationships+xml\" />
	<Default
        Extension=\"texture\"
        ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodeltexture\" />
</Types>
",
    )
}

fn relationships(model_path: &str) -> String {
    format!(
        "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>
<Relationships
    xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">
	<Relationship
        Target=\"/{model_path}\"
        Id=\"rel0\"
        Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\" />
</Relationships>
"
    )
}
