use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

/// A length stored as an integer number of micrometers.
///
/// Construct values with [`Length::um`], [`Length::mm`], [`Length::cm`], or
/// [`Length::m`], or as `100.um()` / `5.mm()` / `2.cm()` / `1.m()` via [`ToLength`].
/// Internal resolution is micrometers (`i128`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Length {
    um: i128,
}

impl Length {
    pub const ZERO: Self = Self { um: 0 };

    const UM_PER_MM: i128 = 1_000;
    const UM_PER_CM: i128 = 10_000;
    const UM_PER_M: i128 = 1_000_000;

    pub const fn um(um: i128) -> Self {
        Self { um }
    }

    pub const fn mm(mm: i64) -> Self {
        Self {
            um: mm as i128 * Self::UM_PER_MM,
        }
    }

    pub const fn cm(cm: i64) -> Self {
        Self {
            um: cm as i128 * Self::UM_PER_CM,
        }
    }

    pub const fn m(m: i64) -> Self {
        Self {
            um: m as i128 * Self::UM_PER_M,
        }
    }

    /// This length in micrometers.
    pub const fn as_um(self) -> i128 {
        self.um
    }

    /// This length in millimeters as `f64`, for tessellation and similar math.
    pub const fn as_mm_f64(self) -> f64 {
        self.um as f64 / Self::UM_PER_MM as f64
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            um: self.um + rhs.um,
        }
    }
}

impl AddAssign for Length {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Length {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            um: self.um - rhs.um,
        }
    }
}

impl SubAssign for Length {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for Length {
    type Output = Self;

    fn neg(self) -> Self {
        Self { um: -self.um }
    }
}

impl Mul<i64> for Length {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self {
        Self {
            um: self.um * rhs as i128,
        }
    }
}

impl Mul<Length> for i64 {
    type Output = Length;

    fn mul(self, rhs: Length) -> Length {
        rhs * self
    }
}

impl Div<i64> for Length {
    type Output = Length;

    fn div(self, rhs: i64) -> Length {
        Length {
            um: self.um / rhs as i128,
        }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.um % Self::UM_PER_MM == 0 {
            write!(f, "{} mm", self.um / Self::UM_PER_MM)
        } else {
            write!(f, "{} µm", self.um)
        }
    }
}

/// Conversion from a bare integer into a [`Length`] with an explicit unit.
///
/// ```
/// use ferrocad::ToLength;
///
/// assert_eq!(5.mm(), ferrocad::Length::mm(5));
/// assert_eq!(100.um(), ferrocad::Length::um(100));
/// ```
pub trait ToLength {
    fn um(self) -> Length;
    fn mm(self) -> Length;
    fn cm(self) -> Length;
    fn m(self) -> Length;
}

impl ToLength for i64 {
    fn um(self) -> Length {
        Length::um(self as i128)
    }

    fn mm(self) -> Length {
        Length::mm(self)
    }

    fn cm(self) -> Length {
        Length::cm(self)
    }

    fn m(self) -> Length {
        Length::m(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_convert_to_micrometers() {
        assert_eq!(1.mm(), 1_000.um());
        assert_eq!(1.cm(), 10.mm());
        assert_eq!(1.m(), 1_000.mm());
        assert_eq!(1.m(), 100.cm());
    }

    #[test]
    fn integer_suffix_constructs_length() {
        assert_eq!(100.um(), Length::um(100));
        assert_eq!(5.mm(), Length::mm(5));
        assert_eq!(2.cm(), Length::cm(2));
        assert_eq!(1.m(), Length::m(1));
    }

    #[test]
    fn arithmetic_stays_in_micrometers() {
        assert_eq!(10.mm() + 2.cm(), 30.mm());
        assert_eq!(1.m() - 1.cm(), 990.mm());
        assert_eq!(10.mm() * 3, 30.mm());
        assert_eq!(2 * 10.mm(), 20.mm());
        assert_eq!(10.mm() / 2, 5.mm());
        assert_eq!(5.mm() / 2, 2_500.um());
        assert_eq!(-4.mm(), Length::mm(-4));
    }

    #[test]
    fn tessellation_uses_millimetres() {
        assert_eq!(1.mm().as_mm_f64(), 1.0);
        assert_eq!(2_500.um().as_mm_f64(), 2.5);
    }
}
