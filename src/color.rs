/// An sRGB color with an optional alpha channel.
///
/// `displaycolor` formats the value for 3MF `basematerials`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// 3MF `displaycolor`: `#RRGGBB` when opaque, `#RRGGBBAA` otherwise.
    pub fn displaycolor(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_rgb_is_hex_without_alpha() {
        assert_eq!(Color::rgb(200, 40, 40).displaycolor(), "#C82828");
    }

    #[test]
    fn translucent_rgba_includes_alpha() {
        assert_eq!(Color::rgba(200, 40, 40, 128).displaycolor(), "#C8282880");
    }
}
