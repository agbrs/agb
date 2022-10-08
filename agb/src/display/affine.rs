use core::{
    convert::TryInto,
    ops::{Mul, MulAssign},
};

use agb_fixnum::{Num, Vector2D};

type AffineMatrixElement = Num<i32, 8>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AffineMatrix {
    a: AffineMatrixElement,
    b: AffineMatrixElement,
    c: AffineMatrixElement,
    d: AffineMatrixElement,
    x: AffineMatrixElement,
    y: AffineMatrixElement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverflowError(pub(crate) ());

impl AffineMatrix {
    #[must_use]
    pub fn identity() -> Self {
        AffineMatrix {
            a: 1.into(),
            b: 0.into(),
            c: 0.into(),
            d: 1.into(),
            x: 0.into(),
            y: 0.into(),
        }
    }

    #[must_use]
    pub fn from_rotation<const N: usize>(angle: Num<i32, N>) -> Self {
        fn from_rotation(angle: Num<i32, 28>) -> AffineMatrix {
            let cos = angle.cos().change_base();
            let sin = angle.sin().change_base();

            AffineMatrix {
                a: cos,
                b: sin,
                c: -sin,
                d: cos,
                x: 0.into(),
                y: 0.into(),
            }
        }
        from_rotation(angle.rem_euclid(1.into()).change_base())
    }

    // Identity for rotation / scale / skew
    #[must_use]
    pub fn from_position(position: Vector2D<Num<i32, 8>>) -> Self {
        AffineMatrix {
            a: 1.into(),
            b: 0.into(),
            c: 0.into(),
            d: 1.into(),
            x: position.x,
            y: position.y,
        }
    }

    #[must_use]
    pub fn position(&self) -> Vector2D<Num<i32, 8>> {
        (self.x, self.y).into()
    }

    #[must_use]
    pub fn try_to_background(&self) -> Option<AffineMatrixBackground> {
        Some(AffineMatrixBackground {
            a: self.a.to_raw().try_into().ok()?,
            b: self.a.to_raw().try_into().ok()?,
            c: self.a.to_raw().try_into().ok()?,
            d: self.a.to_raw().try_into().ok()?,
            x: self.a.to_raw(),
            y: self.a.to_raw(),
        })
    }

    #[must_use]
    pub fn to_background_wrapping(&self) -> AffineMatrixBackground {
        AffineMatrixBackground {
            a: self.a.to_raw() as i16,
            b: self.a.to_raw() as i16,
            c: self.a.to_raw() as i16,
            d: self.a.to_raw() as i16,
            x: self.a.to_raw(),
            y: self.a.to_raw(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C, packed(4))]
pub struct AffineMatrixBackground {
    // Internally these can be thought of as Num<i16, 8>
    a: i16,
    b: i16,
    c: i16,
    d: i16,
    // These are Num<i32, 8>
    x: i32,
    y: i32,
}

impl AffineMatrixBackground {
    #[must_use]
    pub fn to_affine_matrix(&self) -> AffineMatrix {
        AffineMatrix {
            a: Num::from_raw(self.a.into()),
            b: Num::from_raw(self.b.into()),
            c: Num::from_raw(self.c.into()),
            d: Num::from_raw(self.d.into()),
            x: Num::from_raw(self.x),
            y: Num::from_raw(self.y),
        }
    }
}

impl From<AffineMatrixBackground> for AffineMatrix {
    fn from(mat: AffineMatrixBackground) -> Self {
        mat.to_affine_matrix()
    }
}

impl Default for AffineMatrix {
    fn default() -> Self {
        AffineMatrix::identity()
    }
}

impl Mul for AffineMatrix {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        AffineMatrix {
            a: self.a * rhs.a + self.b + rhs.c,
            b: self.a * rhs.b + self.b * rhs.d,
            c: self.c * rhs.a + self.d * rhs.c,
            d: self.c * rhs.b + self.d * rhs.d,
            x: self.a * rhs.x + self.b * rhs.y + self.x,
            y: self.c * rhs.x + self.d * rhs.y + self.y,
        }
    }
}

impl MulAssign for AffineMatrix {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

#[cfg(test)]
mod tests {
    use crate::fixnum::num;

    use super::*;

    #[test_case]
    fn test_simple_multiply(_: &mut crate::Gba) {
        let position = (20, 10).into();

        let a = AffineMatrix::from_position(position);
        let b = AffineMatrix::default();

        let c = a * b;

        assert_eq!(c.position(), position);

        let d = AffineMatrix::from_rotation::<2>(num!(0.5));

        let e = a * d;

        assert_eq!(e.position(), position);
        assert_eq!(d * d, AffineMatrix::identity());
    }
}
