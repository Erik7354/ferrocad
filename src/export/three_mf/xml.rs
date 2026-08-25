use serde::{Deserialize, Serialize};

/// 3MF model document in XML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct Model {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: String,
    #[serde(rename = "@xmlns:m", skip_serializing_if = "Option::is_none")]
    pub xmlns_m: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<Metadata>,
    pub resources: Resources,
    pub build: Build,
    #[serde(rename = "@unit", default)]
    pub unit: Unit,
}

/// Length unit for the model. The default unit is millimeter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Micron,
    #[default]
    Millimeter,
    Centimeter,
    Inch,
    Foot,
    Meter,
}

/// One metadata entry in the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Metadata {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$value")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Resources {
    #[serde(default)]
    pub object: Vec<Object>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "m:basematerials"
    )]
    pub basematerials: Option<Vec<BaseMaterials>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename = "m:basematerials")]
pub struct BaseMaterials {
    #[serde(rename = "@id")]
    pub id: usize,
    #[serde(rename = "m:base")]
    pub base: Vec<Base>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Base {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@displaycolor")]
    pub displaycolor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub struct Object {
    #[serde(rename = "@id")]
    pub id: usize,
    #[serde(rename = "@partnumber", skip_serializing_if = "Option::is_none")]
    pub partnumber: Option<String>,
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@pid", skip_serializing_if = "Option::is_none")]
    pub pid: Option<usize>,
    #[serde(rename = "@pindex", skip_serializing_if = "Option::is_none")]
    pub pindex: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<Mesh>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Components {
    pub component: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    #[serde(rename = "@objectid")]
    pub objectid: usize,
    #[serde(rename = "@transform", skip_serializing_if = "Option::is_none")]
    pub transform: Option<[f64; 12]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Build {
    #[serde(default)]
    pub item: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    #[serde(rename = "@objectid")]
    pub objectid: usize,
    #[serde(rename = "@transform", skip_serializing_if = "Option::is_none")]
    pub transform: Option<[f64; 12]>,
    #[serde(rename = "@partnumber", skip_serializing_if = "Option::is_none")]
    pub partnumber: Option<String>,
}

/// Triangle mesh in a 3MF object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mesh {
    pub vertices: Vertices,
    pub triangles: Triangles,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vertices {
    #[serde(default)]
    pub vertex: Vec<Vertex>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Triangles {
    #[serde(default)]
    pub triangle: Vec<Triangle>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@z")]
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Triangle {
    #[serde(rename = "@v1")]
    pub v1: usize,
    #[serde(rename = "@v2")]
    pub v2: usize,
    #[serde(rename = "@v3")]
    pub v3: usize,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            xmlns: "http://schemas.microsoft.com/3dmanufacturing/core/2015/02".to_owned(),
            xmlns_m: Some(
                "http://schemas.microsoft.com/3dmanufacturing/material/2015/02".to_owned(),
            ),
            metadata: Vec::new(),
            resources: Resources::default(),
            build: Build::default(),
            unit: Unit::default(),
        }
    }
}
