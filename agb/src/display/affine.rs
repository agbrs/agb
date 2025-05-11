#![deny(missing_docs)]
//! # Affine matrices for the Game Boy Advance
//!
//! An affine matrix represents an affine transformation, an affine
//! transformation being one which preserves parallel lines (note that this
//! therefore cannot represent perspective seen in games like Super Mario Kart).
//! Affine matrices are used in two places on the GBA, for affine backgrounds
//! and for affine objects.
//!
//! # Linear Algebra
//! As a matrix, they can be manipulated using linear algebra. The short version
//! of this section is to beware that the matrix is the inverse of the normal
//! transformation matrices.
//!
//! One quick thing to point out at the start as it will become very relevant is
//! that matrix-matrix multiplication is not commutative, meaning swapping the
//! order changes the result, or **A** × **B** ≢ **B** × **A**. However,
//! matrices are, at least in the case they are used here, associative, meaning
//! (**AB**)**C** = **A**(**BC**).
//!
//! ## Normal (wrong on GBA!) transformation matrices
//!
//! As a start, normal transformation matrices will transform a shape from it's
//! original position to it's new position. Generally when people talk about
//! transformation matrices they are talking about them in this sense.
//!
//! > If **A** and **B** are transformation matrices, then matrix **C** = **A**
//! > × **B** represents the transformation **A** performed on **B**, or
//! > alternatively **C** is transformation **B** followed by transformation
//! > **A**.
//!
//! This is not what they represent on the GBA! If you are looking up more
//! information about transformation matrices bear this in mind.
//!
//! ## Correct (on GBA) transformation matrices
//!
//! On the GBA, the affine matrix works the other way around. The GBA wants to
//! know for each pixel what colour it should render, to do this it applies the
//! affine transformation matrix to the pixel it is rendering to lookup correct
//! pixel in the texture.
//!
//! This describes the inverse of the previously given transformation matrices.
//!
//! Above I described the matrix **C** = **A** × **B**, but what the GBA wants
//! is the inverse of **C**, or **C**<sup>-1</sup> = (**AB**)<sup>-1</sup> =
//! **B**<sup>-1</sup> × **A**<sup>-1</sup>. This means that if we have the
//! matrices **I** and **J** in the form the GBA expects then
//!
//! > Transformation **K** = **I** × **J** is the transformation **I** followed
//! > by the transformation **J**.
//!
//! Beware if you are used to the other way around!
//!
//! ## Example, rotation around the centre
//!
//! To rotate something around its centre, you will need to move the thing such
//! that the centre is at (0, 0) and then you can rotate it. After that you can
//! move it where you actually want it.
//!
//! These can be done in the order I stated, **A** = **Move To Origin** ×
//! **Rotate** × **Move to Final Position**. Or in code,
//!
//! ```rust,no_run
//! # #![no_std]
//! # #![no_main]
//! use agb::fixnum::{Vector2D, Num, num};
//! use agb::display::affine::AffineMatrix;
//!
//! # fn foo(_gba: &mut agb::Gba) {
//! // size of our thing is 10 pixels by 10 pixels
//! let size_of_thing: Vector2D<Num<i32, 8>> = (10, 10).into();
//! // rotation by a quarter turn
//! let rotation: Num<i32, 8> = num!(0.25);
//! // the final position
//! let position: Vector2D<Num<i32, 8>> = (100, 100).into();
//!
//! // now lets calculate the final transformation matrix!
//! let a = AffineMatrix::from_translation(-size_of_thing / 2)
//!     * AffineMatrix::from_rotation(rotation)
//!     * AffineMatrix::from_translation(position);
//! # }
//! ```

use core::ops::{Mul, MulAssign};

use agb_fixnum::{Num, Vector2D};

type AffineMatrixElement = Num<i32, 8>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// An affine matrix stored in a way that is efficient for the GBA to perform
/// operations on. This implements multiplication.
///
/// ```txt
/// a b x
/// c d y
/// 0 0 0
/// ```
#[allow(missing_docs)]
pub struct AffineMatrix {
    pub a: AffineMatrixElement,
    pub b: AffineMatrixElement,
    pub c: AffineMatrixElement,
    pub d: AffineMatrixElement,
    pub x: AffineMatrixElement,
    pub y: AffineMatrixElement,
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
        fn from_rotation(angle: Num<i32, 8>) -> AffineMatrix {
            let cos = angle.cos().change_base();
            let sin = angle.sin().change_base();

            // This might look backwards, but the gba does texture mapping, ie a
            // point in screen base is transformed using the matrix to graphics
            // space rather than how you might conventionally think of it.
            AffineMatrix {
                a: cos,
                b: -sin,
                c: sin,
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
            x: -position.x,
            y: -position.y,
        }
    }

    #[must_use]
    /// The position fields of the matrix
    pub fn position(&self) -> Vector2D<Num<i32, 8>> {
        (-self.x, -self.y).into()
    }

    /// Attempts to convert the matrix to one which can be used in affine
    /// objects.
    pub fn try_to_object(&self) -> Result<AffineMatrixObject, OverflowError> {
        Ok(AffineMatrixObject {
            a: self.a.try_change_base().ok_or(OverflowError(()))?,
            b: self.b.try_change_base().ok_or(OverflowError(()))?,
            c: self.c.try_change_base().ok_or(OverflowError(()))?,
            d: self.d.try_change_base().ok_or(OverflowError(()))?,
        })
    }

    #[must_use]
    /// Converts the matrix to one which can be used in affine objects
    /// wrapping any value which is too large to be represented there.
    pub fn to_object_wrapping(&self) -> AffineMatrixObject {
        AffineMatrixObject {
            a: Num::from_raw(self.a.to_raw() as i16),
            b: Num::from_raw(self.b.to_raw() as i16),
            c: Num::from_raw(self.c.to_raw() as i16),
            d: Num::from_raw(self.d.to_raw() as i16),
        }
    }

    #[must_use]
    /// Creates an affine matrix from a given (x, y) scaling. This will scale by
    /// the inverse, ie (2, 2) will produce half the size.
    pub fn from_scale(scale: Vector2D<Num<i32, 8>>) -> AffineMatrix {
        AffineMatrix {
            a: scale.x,
            b: 0.into(),
            c: 0.into(),
            d: scale.y,
            x: 0.into(),
            y: 0.into(),
        }
    }

    #[must_use]
    /// Creates an affine matrix from a given (λ, μ) shearing. This will shear by
    /// the inverse, i.e. (2, 0) shear along the x-axis by 0.5 units per y
    pub fn from_shear(shear: Vector2D<Num<i32, 8>>) -> AffineMatrix {
        AffineMatrix {
            a: shear.x * shear.y + 1,
            b: shear.x,
            c: shear.y,
            d: 1.into(),
            x: 0.into(),
            y: 0.into(),
        }
    }
}

impl Default for AffineMatrix {
    fn default() -> Self {
        AffineMatrix::identity()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C, packed(4))]
/// An affine matrix that can be used in affine objects
///
/// ```txt
/// a b
/// c d
/// ```
#[allow(missing_docs)]
pub struct AffineMatrixObject {
    pub a: Num<i16, 8>,
    pub b: Num<i16, 8>,
    pub c: Num<i16, 8>,
    pub d: Num<i16, 8>,
}

impl Default for AffineMatrixObject {
    fn default() -> Self {
        AffineMatrix::identity().to_object_wrapping()
    }
}

impl TryFrom<AffineMatrix> for AffineMatrixObject {
    type Error = OverflowError;

    fn try_from(value: AffineMatrix) -> Result<Self, Self::Error> {
        value.try_to_object()
    }
}

impl AffineMatrixObject {
    #[must_use]
    /// Converts to the affine matrix that is usable in performing efficient
    /// calculations.
    pub fn to_affine_matrix(&self) -> AffineMatrix {
        AffineMatrix {
            a: self.a.change_base(),
            b: self.b.change_base(),
            c: self.c.change_base(),
            d: self.d.change_base(),
            x: 0.into(),
            y: 0.into(),
        }
    }

    pub(crate) fn components(self) -> [u16; 4] {
        [
            self.a.to_raw() as u16,
            self.b.to_raw() as u16,
            self.c.to_raw() as u16,
            self.d.to_raw() as u16,
        ]
    }
}

impl From<AffineMatrixObject> for AffineMatrix {
    fn from(mat: AffineMatrixObject) -> Self {
        mat.to_affine_matrix()
    }
}

impl Mul for AffineMatrix {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        AffineMatrix {
            a: self.a * rhs.a + self.b * rhs.c,
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
