use std::f64::consts::PI;

/// An angle stored in radians.
///
/// Construct values with [`Angle::deg`] or [`Angle::rad`],
/// or as `90.deg()` / `PI.rad()` via [`ToAngle`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Angle {
    rad: f64,
}

impl Angle {
    pub const ZERO: Self = Self { rad: 0.0 };

    pub const fn rad(rad: f64) -> Self {
        Self { rad }
    }

    pub fn deg(deg: f64) -> Self {
        Self {
            rad: deg * PI / 180.0,
        }
    }

    pub const fn as_rad(self) -> f64 {
        self.rad
    }

    pub fn as_deg(self) -> f64 {
        self.rad * 180.0 / PI
    }
}

/// Conversion from a numeric value into an [`Angle`] with an explicit unit.
///
/// ```
/// use ferrocad::ToAngle;
///
/// assert_eq!(90.deg(), ferrocad::Angle::deg(90.0));
/// ```
pub trait ToAngle {
    fn deg(self) -> Angle;
    fn rad(self) -> Angle;
}

impl ToAngle for f64 {
    fn deg(self) -> Angle {
        Angle::deg(self)
    }

    fn rad(self) -> Angle {
        Angle::rad(self)
    }
}

impl ToAngle for i64 {
    fn deg(self) -> Angle {
        Angle::deg(self as f64)
    }

    fn rad(self) -> Angle {
        Angle::rad(self as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_and_radians_convert() {
        assert!((90.deg().as_rad() - PI / 2.0).abs() < 1e-12);
        assert!((90.deg().as_deg() - 90.0).abs() < 1e-12);
        assert_eq!(1.rad(), Angle::rad(1.0));
    }
}
