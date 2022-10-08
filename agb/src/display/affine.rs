#![deny(missing_docs)]
//! # Affine matricies for the Game Boy Advance
//!
//! An affine matrix represents an affine transformation, an affine
//! transformation being one which preserves parallel lines (note that this
//! therefore cannot represent perspective seen in games like Super Mario Kart).
//! Affine matricies are used in two places on the GBA, for affine backgrounds
//! and for affine objects.
//!
//! # Linear Algebra basics
//! As a matrix, they can be manipulated using linear algebra, although you
//! shouldn't need to know linear algebra to use this apart from a few things
//!
//! If `A` and `B` are matricies, then matrix `C = A * B` represents the
//! transformation `A` performed on `B`, or alternatively `C` is transformation
//! `B` followed by transformation `A`.
//!
//! Additionally matrix multiplication is not commutative, meaning swapping the
//! order changes the result, or `A * B ≢ B * A`.

use core::{
    convert::{TryFrom, TryInto},
    ops::{Mul, MulAssign},
};

use agb_fixnum::{Num, Vector2D};

type AffineMatrixElement = Num<i32, 8>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// An affine matrix stored in a way that is efficient for the GBA to perform
/// operations on. This implements multiplication.
pub struct AffineMatrix {
    a: AffineMatrixElement,
    b: AffineMatrixElement,
    c: AffineMatrixElement,
    d: AffineMatrixElement,
    x: AffineMatrixElement,
    y: AffineMatrixElement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The error emitted upon a conversion that could not be performed due to
/// overflowing the destination data size
pub struct OverflowError(pub(crate) ());

impl AffineMatrix {
    #[must_use]
    /// The Identity matrix. The identity matrix can be thought of as 1 and is
    /// represented by `I`. For a matrix `A`, `A ≡ A * I ≡ I * A`.
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
    /// Generates the matrix that represents a rotation
    pub fn from_rotation<const N: usize>(angle: Num<i32, N>) -> Self {
        fn from_rotation(angle: Num<i32, 28>) -> AffineMatrix {
            let cos = angle.cos().change_base();
            let sin = angle.sin().change_base();

            // This might look backwards, but the gba does texture mapping, ie a
            // point in screen base is transformed using the matrix to graphics
            // space rather than how you might conventionally think of it.
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
    /// Generates the matrix that represents a translation by the position
    #[must_use]
    pub fn from_translation(position: Vector2D<Num<i32, 8>>) -> Self {
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
    /// The position fields of the matrix
    pub fn position(&self) -> Vector2D<Num<i32, 8>> {
        (self.x, self.y).into()
    }

    /// Attempts to convert the matrix to one which can be used in affine
    /// backgrounds.
    pub fn try_to_background(&self) -> Result<AffineMatrixBackground, OverflowError> {
        Ok(AffineMatrixBackground {
            a: self.a.to_raw().try_into().map_err(|_| OverflowError(()))?,
            b: self.a.to_raw().try_into().map_err(|_| OverflowError(()))?,
            c: self.a.to_raw().try_into().map_err(|_| OverflowError(()))?,
            d: self.a.to_raw().try_into().map_err(|_| OverflowError(()))?,
            x: self.a.to_raw(),
            y: self.a.to_raw(),
        })
    }

    #[must_use]
    /// Converts the matrix to one which can be used in affine backgrounds
    /// wrapping any value which is too large to be represented there.
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
/// An affine matrix that can be used in affine backgrounds
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

impl TryFrom<AffineMatrix> for AffineMatrixBackground {
    type Error = OverflowError;

    fn try_from(value: AffineMatrix) -> Result<Self, Self::Error> {
        value.try_to_background()
    }
}

impl AffineMatrixBackground {
    #[must_use]
    /// Converts to the affine matrix that is usable in performing efficient
    /// calculations.
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

        let a = AffineMatrix::from_translation(position);
        let b = AffineMatrix::default();

        let c = a * b;

        assert_eq!(c.position(), position);

        let d = AffineMatrix::from_rotation::<2>(num!(0.5));

        let e = a * d;

        assert_eq!(e.position(), position);
        assert_eq!(d * d, AffineMatrix::identity());
    }
}
