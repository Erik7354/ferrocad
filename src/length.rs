use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

/// A length stored as an integer number of millimeters.
///
/// Construct values with [`Length::mm`], [`Length::cm`], or [`Length::m`],
/// or as `5.mm()` / `2.cm()` / `1.m()` via [`ToLength`].
/// Internal resolution is millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Length {
    mm: i64,
}

impl Length {
    pub const ZERO: Self = Self { mm: 0 };

    pub const fn mm(mm: i64) -> Self {
        Self { mm }
    }

    pub const fn cm(cm: i64) -> Self {
        Self { mm: cm * 10 }
    }

    pub const fn m(m: i64) -> Self {
        Self { mm: m * 1_000 }
    }

    /// This length in millimeters.
    pub const fn as_mm(self) -> i64 {
        self.mm
    }

    /// This length in millimeters as `f64`, for tessellation and similar math.
    pub const fn as_mm_f64(self) -> f64 {
        self.mm as f64
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            mm: self.mm + rhs.mm,
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
            mm: self.mm - rhs.mm,
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
        Self { mm: -self.mm }
    }
}

impl Mul<i64> for Length {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self {
        Self { mm: self.mm * rhs }
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
        Length { mm: self.mm / rhs }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} mm", self.mm)
    }
}

/// Conversion from a bare integer into a [`Length`] with an explicit unit.
///
/// ```
/// use ferrocad::ToLength;
///
/// assert_eq!(5.mm(), ferrocad::Length::mm(5));
/// ```
pub trait ToLength {
    fn mm(self) -> Length;
    fn cm(self) -> Length;
    fn m(self) -> Length;
}

impl ToLength for i64 {
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
    fn units_convert_to_millimeters() {
        assert_eq!(1.cm(), 10.mm());
        assert_eq!(1.m(), 1_000.mm());
        assert_eq!(1.m(), 100.cm());
    }

    #[test]
    fn integer_suffix_constructs_length() {
        assert_eq!(5.mm(), Length::mm(5));
        assert_eq!(2.cm(), Length::cm(2));
        assert_eq!(1.m(), Length::m(1));
    }

    #[test]
    fn arithmetic_stays_in_millimeters() {
        assert_eq!(10.mm() + 2.cm(), 30.mm());
        assert_eq!(1.m() - 1.cm(), 990.mm());
        assert_eq!(10.mm() * 3, 30.mm());
        assert_eq!(2 * 10.mm(), 20.mm());
        assert_eq!(10.mm() / 2, 5.mm());
        assert_eq!(-4.mm(), Length::mm(-4));
    }
}
