use crate::Length;
use crate::color::Color;
use crate::mesh::Mesh;

/// A printable assembly: named, optionally colored meshes.
#[derive(Debug, Clone, Default)]
pub struct Model {
    objects: Vec<Object>,
    print: PrintSettings,
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
    pub print: PrintSettings,
}

impl Object {
    pub fn new(name: impl Into<String>, mesh: Mesh) -> Self {
        Self {
            name: name.into(),
            mesh,
            color: None,
            print: PrintSettings::default(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_print(mut self, print: PrintSettings) -> Self {
        self.print = print;
        self
    }
}

/// Print intent for a model or one object.
///
/// Each field is optional. A `None` value inherits from the parent
/// (object from model, model from the slicer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PrintSettings {
    pub nozzle: Option<Nozzle>,
    pub layer_height: Option<Length>,
    pub infill: Option<Infill>,
    pub support: Option<Support>,
    pub walls: Option<u32>,
}

/// Common FDM nozzle diameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nozzle {
    /// 0.2 mm
    D02,
    /// 0.4 mm
    D04,
    /// 0.6 mm
    D06,
    /// 0.8 mm
    D08,
}

impl Nozzle {
    pub fn diameter(self) -> Length {
        match self {
            Self::D02 => Length::um(200),
            Self::D04 => Length::um(400),
            Self::D06 => Length::um(600),
            Self::D08 => Length::um(800),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Infill {
    pub pattern: Option<InfillPattern>,
    /// Percent, 0 through 100.
    pub density: Option<u8>,
}

/// Patterns that Bambu and Prusa both know.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfillPattern {
    Lines,
    Grid,
    Gyroid,
    Honeycomb,
    Cubic,
    Lightning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Support {
    Off,
    On {
        kind: SupportKind,
        on_build_plate_only: bool,
        support_critical_regions_only: bool,
        /// Gap between support and part. Maps to slicer contact distance.
        z_offset: Option<Length>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SupportKind {
    #[default]
    Normal,
    Tree,
}
