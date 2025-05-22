use core::ops::{Mul, MulAssign};

use crate::fixnum::{FixedWidthSignedInteger, Num, SignedNumber, Vector2D, num, vec2};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// An affine matrix stored in a way that is efficient for the GBA to perform
/// operations on. This implements multiplication.
///
/// ```txt
/// a b x
/// c d y
/// 0 0 0
/// ```
///
/// # Affine matrices for the Game Boy Advance
///
/// An affine matrix represents an affine transformation, an affine
/// transformation being one which preserves parallel lines (note that this
/// therefore cannot represent perspective seen in games like Super Mario Kart).
/// Affine matrices are used in two places on the GBA, for affine backgrounds
/// and for affine objects.
///
/// # Linear Algebra
/// As a matrix, they can be manipulated using linear algebra. The short version
/// of this section is to beware that the matrix is the inverse of the normal
/// transformation matrices.
///
/// One quick thing to point out at the start as it will become very relevant is
/// that matrix-matrix multiplication is not commutative, meaning swapping the
/// order changes the result, or **A** × **B** ≢ **B** × **A**. However,
/// matrices are, at least in the case they are used here, associative, meaning
/// (**AB**)**C** = **A**(**BC**).
///
/// ## Normal (wrong on GBA!) transformation matrices
///
/// As a start, normal transformation matrices will transform a shape from it's
/// original position to it's new position. Generally when people talk about
/// transformation matrices they are talking about them in this sense.
///
/// > If **A** and **B** are transformation matrices, then matrix **C** = **A**
/// > × **B** represents the transformation **A** performed on **B**, or
/// > alternatively **C** is transformation **B** followed by transformation
/// > **A**.
///
/// This is not what they represent on the GBA! If you are looking up more
/// information about transformation matrices bear this in mind.
///
/// ## Correct (on GBA) transformation matrices
///
/// On the GBA, the affine matrix works the other way around. The GBA wants to
/// know for each pixel what colour it should render, to do this it applies the
/// affine transformation matrix to the pixel it is rendering to lookup correct
/// pixel in the texture.
///
/// This describes the inverse of the previously given transformation matrices.
///
/// Above I described the matrix **C** = **A** × **B**, but what the GBA wants
/// is the inverse of **C**, or **C**<sup>-1</sup> = (**AB**)<sup>-1</sup> =
/// **B**<sup>-1</sup> × **A**<sup>-1</sup>. This means that if we have the
/// matrices **I** and **J** in the form the GBA expects then
///
/// > Transformation **K** = **I** × **J** is the transformation **I** followed
/// > by the transformation **J**.
///
/// Beware if you are used to the other way around!
///
/// ## Example, rotation around the centre
///
/// To rotate something around its centre, you will need to move the thing such
/// that the centre is at (0, 0) and then you can rotate it. After that you can
/// move it where you actually want it.
///
/// These can be done in the order I stated, **A** = **Move To Origin** ×
/// **Rotate** × **Move to Final Position**. Or in code,
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// use agb::fixnum::{Vector2D, Num, num};
/// use agb::display::AffineMatrix;
///
/// # fn foo(_gba: &mut agb::Gba) {
/// // size of our thing is 10 pixels by 10 pixels
/// let size_of_thing: Vector2D<Num<i32, 8>> = (10, 10).into();
/// // rotation by a quarter turn
/// let rotation: Num<i32, 8> = num!(0.25);
/// // the final position
/// let position: Vector2D<Num<i32, 8>> = (100, 100).into();
///
/// // now lets calculate the final transformation matrix!
/// let a = AffineMatrix::from_translation(-size_of_thing / 2)
///     * AffineMatrix::from_rotation(rotation)
///     * AffineMatrix::from_translation(position);
/// # }
/// ```
#[allow(missing_docs)]
pub struct AffineMatrix<T> {
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
    pub x: T,
    pub y: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The error emitted upon a conversion that could not be performed due to
/// overflowing the destination data size
pub struct OverflowError(pub(crate) ());

impl<T: SignedNumber> AffineMatrix<T> {
    #[must_use]
    /// The Identity matrix. The identity matrix can be thought of as 1 and is
    /// represented by `I`. For a matrix `A`, `A ≡ A * I ≡ I * A`.
    pub fn identity() -> Self {
        AffineMatrix {
            a: T::one(),
            b: T::zero(),
            c: T::zero(),
            d: T::one(),
            x: T::zero(),
            y: T::zero(),
        }
    }

    // Identity for rotation / scale / skew
    /// Generates the matrix that represents a translation by the position
    #[must_use]
    pub fn from_translation(position: Vector2D<T>) -> Self {
        AffineMatrix {
            a: T::one(),
            b: T::zero(),
            c: T::zero(),
            d: T::one(),
            x: -position.x,
            y: -position.y,
        }
    }

    #[must_use]
    /// The position fields of the matrix
    pub fn position(&self) -> Vector2D<T> {
        vec2(-self.x, -self.y)
    }

    #[must_use]
    /// Creates an affine matrix from a given (x, y) scaling. This will scale by
    /// the inverse, ie (2, 2) will produce half the size.
    pub fn from_scale(scale: Vector2D<T>) -> Self {
        Self {
            a: scale.x,
            b: T::zero(),
            c: T::zero(),
            d: scale.y,
            x: T::zero(),
            y: T::zero(),
        }
    }

    #[must_use]
    /// Creates an affine matrix from a given (λ, μ) shearing.
    pub fn from_shear(shear: Vector2D<T>) -> AffineMatrix<T> {
        AffineMatrix {
            a: shear.x * shear.y + T::one(),
            b: shear.x,
            c: shear.y,
            d: T::one(),
            x: T::zero(),
            y: T::zero(),
        }
    }
}

impl<I, const N: usize> AffineMatrix<Num<I, N>>
where
    I: FixedWidthSignedInteger,
{
    #[must_use]
    /// Generates the matrix that represents a rotation
    pub fn from_rotation(angle: Num<I, N>) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();

        // This might look backwards, but the gba does texture mapping, ie a
        // point in screen base is transformed using the matrix to graphics
        // space rather than how you might conventionally think of it.
        AffineMatrix {
            a: cos,
            b: -sin,
            c: sin,
            d: cos,
            x: num!(0),
            y: num!(0),
        }
    }

    /// Change from one `Num` kind to another where the conversion is loss-less
    #[must_use]
    pub fn change_base<J, const M: usize>(self) -> AffineMatrix<Num<J, M>>
    where
        J: FixedWidthSignedInteger + From<I>,
    {
        AffineMatrix {
            a: self.a.change_base(),
            b: self.b.change_base(),
            c: self.c.change_base(),
            d: self.d.change_base(),
            x: self.x.change_base(),
            y: self.y.change_base(),
        }
    }
}

impl<T: SignedNumber> Default for AffineMatrix<T> {
    fn default() -> Self {
        AffineMatrix::identity()
    }
}

impl<T: SignedNumber> Mul for AffineMatrix<T> {
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

impl<T: SignedNumber> MulAssign for AffineMatrix<T> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T: SignedNumber> Mul<Vector2D<T>> for AffineMatrix<T> {
    type Output = Vector2D<T>;

    fn mul(self, rhs: Vector2D<T>) -> Self::Output {
        vec2(
            self.a * rhs.x + self.b * rhs.y + self.x,
            self.c * rhs.x + self.d * rhs.y + self.y,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::fixnum::num;

    use super::*;

    #[test_case]
    fn test_simple_multiply(_: &mut crate::Gba) {
        let position: Vector2D<Num<i32, 8>> = (20, 10).into();

        let a = AffineMatrix::from_translation(position);
        let b = AffineMatrix::default();

        let c = a * b;

        assert_eq!(c.position(), position);

        let d = AffineMatrix::from_rotation(num!(0.5));

        let e = a * d;

        assert_eq!(e.position(), position);
        assert_eq!(d * d, AffineMatrix::identity());
    }
}
