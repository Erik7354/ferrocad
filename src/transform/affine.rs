use std::ops::Mul;

use crate::angle::Angle;
use crate::length::Length;
use crate::mesh::{Point2, Point3};

/// A 2D affine map, stored as a 3×3 matrix in homogeneous coordinates.
///
/// A 2×2 matrix is linear: rotation, scale, shear, reflection. It always
/// fixes the origin, so it cannot represent translation. Affine maps are
/// `p ↦ Ap + t`. Homogeneous points `(x, y, 1)` turn that into one
/// multiply:
///
/// ```text
/// [ x' ]   [ a  b  tx ] [ x ]
/// [ y' ] = [ c  d  ty ] [ y ]
/// [  1 ]   [ 0  0   1 ] [ 1 ]
/// ```
///
/// The upper-left 2×2 is the linear part; the third column is translation.
/// The last row is always `[0, 0, 1]` for a proper affine map (it is not a
/// general projective 3×3). A 2×3, or a 2×2 plus `tx`/`ty`, stores the same
/// six numbers; the extra row is kept so composition is ordinary matrix
/// multiply, matching [`Affine3`]'s 4×4.
///
/// Points are column vectors; `A * B` means apply `B` first, then `A`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine2 {
    m: [[f64; 3]; 3],
}

impl Affine2 {
    pub const IDENTITY: Self = Self {
        m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    /// Build a map from a 3×3 matrix. The last row is set to `[0, 0, 1]`.
    pub const fn from_rows(mut rows: [[f64; 3]; 3]) -> Self {
        rows[2] = [0.0, 0.0, 1.0];
        Self { m: rows }
    }

    pub const fn rows(self) -> [[f64; 3]; 3] {
        self.m
    }

    pub const fn translate(x: Length, y: Length) -> Self {
        Self::from_rows([
            [1.0, 0.0, x.as_mm_f64()],
            [0.0, 1.0, y.as_mm_f64()],
            [0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate(angle: Angle) -> Self {
        let c = angle.as_rad().cos();
        let s = angle.as_rad().sin();
        Self::from_rows([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])
    }

    pub const fn scale(x: f64, y: f64) -> Self {
        Self::from_rows([[x, 0.0, 0.0], [0.0, y, 0.0], [0.0, 0.0, 1.0]])
    }

    /// Reflection through the line through the origin with the given normal.
    pub fn mirror(x: f64, y: f64) -> Self {
        let n2 = x * x + y * y;
        if n2 == 0.0 {
            return Self::IDENTITY;
        }
        let f = 2.0 / n2;
        Self::from_rows([
            [1.0 - f * x * x, -f * x * y, 0.0],
            [-f * y * x, 1.0 - f * y * y, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// Determinant of the linear (upper-left 2×2) part.
    pub fn determinant(self) -> f64 {
        self.m[0][0] * self.m[1][1] - self.m[0][1] * self.m[1][0]
    }

    pub fn apply(self, point: Point2) -> Point2 {
        Point2::new(
            self.m[0][0] * point.x + self.m[0][1] * point.y + self.m[0][2],
            self.m[1][0] * point.x + self.m[1][1] * point.y + self.m[1][2],
        )
    }
}

impl Default for Affine2 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Affine2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut m = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] = self.m[i][0] * rhs.m[0][j]
                    + self.m[i][1] * rhs.m[1][j]
                    + self.m[i][2] * rhs.m[2][j];
            }
        }
        Self { m }
    }
}

impl Mul<Point2> for Affine2 {
    type Output = Point2;

    fn mul(self, rhs: Point2) -> Point2 {
        self.apply(rhs)
    }
}

/// A 3D affine map, stored as a 4×4 matrix in homogeneous coordinates.
///
/// A 3×3 matrix is linear and cannot translate. Affine maps are `p ↦ Ap + t`.
/// Homogeneous points `(x, y, z, 1)` turn that into one multiply:
///
/// ```text
/// [ x' ]   [ R | t ] [ x ]
/// [ y' ] = [   |   ] [ y ]
/// [ z' ]   [ 0 | 1 ] [ z ]
/// [  1 ]             [ 1 ]
/// ```
///
/// The upper-left 3×3 is the linear part; the fourth column is translation.
/// The last row is always `[0, 0, 0, 1]`. A 3×4 stores the same numbers; the
/// extra row is kept so composition is ordinary matrix multiply, matching
/// [`Affine2`]'s 3×3.
///
/// Points are column vectors; `A * B` means apply `B` first, then `A`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine3 {
    m: [[f64; 4]; 4],
}

impl Affine3 {
    pub const IDENTITY: Self = Self {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// Build a map from a 4×4 matrix. The last row is set to `[0, 0, 0, 1]`.
    pub const fn from_rows(mut rows: [[f64; 4]; 4]) -> Self {
        rows[3] = [0.0, 0.0, 0.0, 1.0];
        Self { m: rows }
    }

    pub const fn rows(self) -> [[f64; 4]; 4] {
        self.m
    }

    pub const fn translate(x: Length, y: Length, z: Length) -> Self {
        Self::from_rows([
            [1.0, 0.0, 0.0, x.as_mm_f64()],
            [0.0, 1.0, 0.0, y.as_mm_f64()],
            [0.0, 0.0, 1.0, z.as_mm_f64()],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Rotation by Euler angles around X, then Y, then Z.
    pub fn rotate(x: Angle, y: Angle, z: Angle) -> Self {
        Self::rotate_z(z) * Self::rotate_y(y) * Self::rotate_x(x)
    }

    pub fn rotate_x(angle: Angle) -> Self {
        let c = angle.as_rad().cos();
        let s = angle.as_rad().sin();
        Self::from_rows([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, -s, 0.0],
            [0.0, s, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate_y(angle: Angle) -> Self {
        let c = angle.as_rad().cos();
        let s = angle.as_rad().sin();
        Self::from_rows([
            [c, 0.0, s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate_z(angle: Angle) -> Self {
        let c = angle.as_rad().cos();
        let s = angle.as_rad().sin();
        Self::from_rows([
            [c, -s, 0.0, 0.0],
            [s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Rotation by `angle` around the vector `(x, y, z)`.
    pub fn rotate_axis(angle: Angle, x: f64, y: f64, z: f64) -> Self {
        let len = (x * x + y * y + z * z).sqrt();
        if len == 0.0 {
            return Self::IDENTITY;
        }
        let (x, y, z) = (x / len, y / len, z / len);
        let c = angle.as_rad().cos();
        let s = angle.as_rad().sin();
        let t = 1.0 - c;
        Self::from_rows([
            [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
            [t * y * x + s * z, t * y * y + c, t * y * z - s * x, 0.0],
            [t * z * x - s * y, t * z * y + s * x, t * z * z + c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub const fn scale(x: f64, y: f64, z: f64) -> Self {
        Self::from_rows([
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Reflection through the plane through the origin with the given normal.
    pub fn mirror(x: f64, y: f64, z: f64) -> Self {
        let n2 = x * x + y * y + z * z;
        if n2 == 0.0 {
            return Self::IDENTITY;
        }
        let f = 2.0 / n2;
        Self::from_rows([
            [1.0 - f * x * x, -f * x * y, -f * x * z, 0.0],
            [-f * y * x, 1.0 - f * y * y, -f * y * z, 0.0],
            [-f * z * x, -f * z * y, 1.0 - f * z * z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Determinant of the linear (upper-left 3×3) part.
    pub fn determinant(self) -> f64 {
        let a = self.m[0][0];
        let b = self.m[0][1];
        let c = self.m[0][2];
        let d = self.m[1][0];
        let e = self.m[1][1];
        let f = self.m[1][2];
        let g = self.m[2][0];
        let h = self.m[2][1];
        let i = self.m[2][2];
        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    pub fn apply(self, point: Point3) -> Point3 {
        Point3::new(
            self.m[0][0] * point.x + self.m[0][1] * point.y + self.m[0][2] * point.z + self.m[0][3],
            self.m[1][0] * point.x + self.m[1][1] * point.y + self.m[1][2] * point.z + self.m[1][3],
            self.m[2][0] * point.x + self.m[2][1] * point.y + self.m[2][2] * point.z + self.m[2][3],
        )
    }
}

impl Default for Affine3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Affine3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut m = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                m[i][j] = self.m[i][0] * rhs.m[0][j]
                    + self.m[i][1] * rhs.m[1][j]
                    + self.m[i][2] * rhs.m[2][j]
                    + self.m[i][3] * rhs.m[3][j];
            }
        }
        Self { m }
    }
}

impl Mul<Point3> for Affine3 {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Point3 {
        self.apply(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToAngle;
    use crate::ToLength;

    fn assert_point2(actual: Point2, expected: Point2) {
        assert!(
            (actual.x - expected.x).abs() < 1e-9 && (actual.y - expected.y).abs() < 1e-9,
            "{actual:?} != {expected:?}"
        );
    }

    fn assert_point3(actual: Point3, expected: Point3) {
        assert!(
            (actual.x - expected.x).abs() < 1e-9
                && (actual.y - expected.y).abs() < 1e-9
                && (actual.z - expected.z).abs() < 1e-9,
            "{actual:?} != {expected:?}"
        );
    }

    #[test]
    fn translate_moves_a_point() {
        assert_point3(
            Affine3::translate(10.mm(), -4.mm(), 2.mm()) * Point3::new(1.0, 2.0, 3.0),
            Point3::new(11.0, -2.0, 5.0),
        );
        assert_point2(
            Affine2::translate(10.mm(), -4.mm()) * Point2::new(1.0, 2.0),
            Point2::new(11.0, -2.0),
        );
    }

    #[test]
    fn rotate_z_90_sends_x_to_y() {
        assert_point3(
            Affine3::rotate(Angle::ZERO, Angle::ZERO, 90.deg()) * Point3::new(10.0, 0.0, 3.0),
            Point3::new(0.0, 10.0, 3.0),
        );
        assert_point3(
            Affine3::rotate_axis(90.deg(), 0.0, 0.0, 1.0) * Point3::new(10.0, 0.0, 3.0),
            Point3::new(0.0, 10.0, 3.0),
        );
        assert_point2(
            Affine2::rotate(90.deg()) * Point2::new(10.0, 0.0),
            Point2::new(0.0, 10.0),
        );
    }

    #[test]
    fn scale_and_mirror() {
        assert_point3(
            Affine3::scale(2.0, 3.0, 4.0) * Point3::new(1.0, 1.0, 1.0),
            Point3::new(2.0, 3.0, 4.0),
        );
        assert_point3(
            Affine3::mirror(1.0, 0.0, 0.0) * Point3::new(5.0, 2.0, 3.0),
            Point3::new(-5.0, 2.0, 3.0),
        );
        assert!(Affine3::mirror(1.0, 0.0, 0.0).determinant() < 0.0);
        assert!(Affine3::scale(2.0, 2.0, 2.0).determinant() > 0.0);
    }

    #[test]
    fn composition_applies_right_factor_first() {
        let translate = Affine3::translate(10.mm(), 0.mm(), 0.mm());
        let rotate = Affine3::rotate_z(90.deg());
        let origin = Point3::new(0.0, 0.0, 0.0);

        assert_point3(rotate * translate * origin, Point3::new(0.0, 10.0, 0.0));
        assert_point3(translate * rotate * origin, Point3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn rotate_x_90_sends_y_to_z() {
        assert_point3(
            Affine3::rotate(90.deg(), Angle::ZERO, Angle::ZERO) * Point3::new(0.0, 10.0, 0.0),
            Point3::new(0.0, 0.0, 10.0),
        );
    }

    #[test]
    fn rotate_y_90_sends_z_to_x() {
        assert_point3(
            Affine3::rotate(Angle::ZERO, 90.deg(), Angle::ZERO) * Point3::new(0.0, 0.0, 10.0),
            Point3::new(10.0, 0.0, 0.0),
        );
    }

    #[test]
    fn from_rows_forces_an_affine_last_row() {
        let a2 = Affine2::from_rows([[1.0, 0.0, 5.0], [0.0, 1.0, 0.0], [0.1, 0.0, 2.0]]);
        assert_eq!(a2.rows()[2], [0.0, 0.0, 1.0]);
        assert_point2(a2 * Point2::new(1.0, 0.0), Point2::new(6.0, 0.0));

        let a3 = Affine3::from_rows([
            [1.0, 0.0, 0.0, 5.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.1, 0.0, 0.0, 2.0],
        ]);
        assert_eq!(a3.rows()[3], [0.0, 0.0, 0.0, 1.0]);
        assert_point3(a3 * Point3::new(1.0, 0.0, 0.0), Point3::new(6.0, 0.0, 0.0));
    }
}
