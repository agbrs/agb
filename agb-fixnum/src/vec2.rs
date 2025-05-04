use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use num_traits::Signed;

use crate::{FixedWidthSignedInteger, FixedWidthUnsignedInteger, Num, Number, num};

/// A vector of two points: (x, y) represented by integers or fixed point numbers
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Vector2D<T: Number> {
    /// The x coordinate
    pub x: T,
    /// The y coordinate
    pub y: T,
}

/// A convenience function for constructing a Vector2D
///
/// ```
/// use agb_fixnum::{vec2, Vector2D};
///
/// assert_eq!(vec2(3, 5), Vector2D::new(3, 5));
/// ```
pub const fn vec2<T: Number>(x: T, y: T) -> Vector2D<T> {
    Vector2D::new(x, y)
}

impl<T: Number> Add<Vector2D<T>> for Vector2D<T> {
    type Output = Vector2D<T>;
    fn add(self, rhs: Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Number, U: Copy> Mul<U> for Vector2D<T>
where
    T: Mul<U, Output = T>,
{
    type Output = Vector2D<T>;
    fn mul(self, rhs: U) -> Self::Output {
        Vector2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Number, U: Copy> MulAssign<U> for Vector2D<T>
where
    T: Mul<U, Output = T>,
{
    fn mul_assign(&mut self, rhs: U) {
        let result = *self * rhs;
        self.x = result.x;
        self.y = result.y;
    }
}

impl<T: Number, U: Copy> Div<U> for Vector2D<T>
where
    T: Div<U, Output = T>,
{
    type Output = Vector2D<T>;
    fn div(self, rhs: U) -> Self::Output {
        Vector2D {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Number, U: Copy> DivAssign<U> for Vector2D<T>
where
    T: Div<U, Output = T>,
{
    fn div_assign(&mut self, rhs: U) {
        let result = *self / rhs;
        self.x = result.x;
        self.y = result.y;
    }
}

impl<T: Number> AddAssign<Self> for Vector2D<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: Number> Sub<Vector2D<T>> for Vector2D<T> {
    type Output = Vector2D<T>;
    fn sub(self, rhs: Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Number> SubAssign<Self> for Vector2D<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: Number + Signed> Vector2D<T> {
    /// Calculates the absolute value of the x and y components.
    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    #[must_use]
    /// Calculates the manhattan distance, x.abs() + y.abs().
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(3.), num!(4.)).into();
    /// assert_eq!(v1.manhattan_distance(), 7.into());
    /// ```
    pub fn manhattan_distance(self) -> T {
        self.x.abs() + self.y.abs()
    }
}

impl<I: FixedWidthUnsignedInteger, const N: usize> Vector2D<Num<I, N>> {
    #[must_use]
    /// Truncates the x and y coordinate, see [`Num::trunc`]
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(1.56), num!(-2.2)).into();
    /// let v2: Vector2D<i32> = (1, -2).into();
    /// assert_eq!(v1.trunc(), v2);
    /// ```
    pub fn trunc(self) -> Vector2D<I> {
        Vector2D {
            x: self.x.trunc(),
            y: self.y.trunc(),
        }
    }

    #[must_use]
    /// Floors the x and y coordinate, see [Num::floor]
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = Vector2D::new(num!(1.56), num!(-2.2));
    /// let v2: Vector2D<i32> = (1, -3).into();
    /// assert_eq!(v1.floor(), v2);
    /// ```
    pub fn floor(self) -> Vector2D<I> {
        Vector2D {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    #[must_use]
    /// Rounds the x and y coordinate, see [Num::round]
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = Vector2D::new(num!(1.56), num!(-2.2));
    /// let v2: Vector2D<i32> = (2, -2).into();
    /// assert_eq!(v1.round(), v2);
    /// ```
    pub fn round(self) -> Vector2D<I> {
        Vector2D {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    #[must_use]
    /// Attempts to change the base returning None if the numbers cannot be represented
    pub fn try_change_base<J: FixedWidthUnsignedInteger + TryFrom<I>, const M: usize>(
        self,
    ) -> Option<Vector2D<Num<J, M>>> {
        Some(Vector2D::new(
            self.x.try_change_base()?,
            self.y.try_change_base()?,
        ))
    }
}

impl<const N: usize> Vector2D<Num<i32, N>> {
    #[must_use]
    /// Calculates the magnitude by square root
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(3.), num!(4.)).into();
    /// assert_eq!(v1.magnitude(), 5.into());
    /// ```
    pub fn magnitude(self) -> Num<i32, N> {
        self.magnitude_squared().sqrt()
    }

    /// Calculates the magnitude of a vector using the [alpha max plus beta min
    /// algorithm](https://en.wikipedia.org/wiki/Alpha_max_plus_beta_min_algorithm)
    /// this has a maximum error of less than 4% of the true magnitude, probably
    /// depending on the size of your fixed point approximation
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(3.), num!(4.)).into();
    /// assert!(v1.fast_magnitude() > num!(4.9) && v1.fast_magnitude() < num!(5.1));
    /// ```
    #[must_use]
    pub fn fast_magnitude(self) -> Num<i32, N> {
        let max = core::cmp::max(self.x.abs(), self.y.abs());
        let min = core::cmp::min(self.x.abs(), self.y.abs());

        max * num!(0.960433870103) + min * num!(0.397824734759)
    }

    #[must_use]
    /// Normalises the vector to magnitude of one by performing a square root,
    /// due to fixed point imprecision this magnitude may not be exactly one
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(4.), num!(4.)).into();
    /// assert_eq!(v1.normalise().magnitude(), 1.into());
    /// ```
    pub fn normalise(self) -> Self {
        self / self.magnitude()
    }

    #[must_use]
    /// Normalises the vector to magnitude of one using [Vector2D::fast_magnitude].
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(4.), num!(4.)).into();
    /// assert_eq!(v1.fast_normalise().magnitude(), 1.into());
    /// ```
    pub fn fast_normalise(self) -> Self {
        self / self.fast_magnitude()
    }
}

impl<T: Number, P: Number + Into<T>> From<(P, P)> for Vector2D<T> {
    fn from(f: (P, P)) -> Self {
        Vector2D::new(f.0.into(), f.1.into())
    }
}

impl<T: Number> Vector2D<T> {
    /// Converts the representation of the vector to another type
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<i16> = Vector2D::new(1, 2);
    /// let v2: Vector2D<i32> = v1.change_base();
    /// ```
    pub fn change_base<U: Number + From<T>>(self) -> Vector2D<U> {
        (self.x, self.y).into()
    }
}

impl<I: FixedWidthSignedInteger, const N: usize> Vector2D<Num<I, N>> {
    /// Creates a unit vector from an angle, noting that the domain of the angle
    /// is [0, 1], see [Num::cos] and [Num::sin].
    /// ```
    /// # use agb_fixnum::*;
    /// let v: Vector2D<Num<i32, 8>> = Vector2D::new_from_angle(num!(0.0));
    /// assert_eq!(v, (num!(1.0), num!(0.0)).into());
    /// ```
    pub fn new_from_angle(angle: Num<I, N>) -> Self {
        Vector2D {
            x: angle.cos(),
            y: angle.sin(),
        }
    }
}

impl<I: FixedWidthUnsignedInteger, const N: usize> From<Vector2D<I>> for Vector2D<Num<I, N>> {
    fn from(n: Vector2D<I>) -> Self {
        Vector2D {
            x: n.x.into(),
            y: n.y.into(),
        }
    }
}

impl<T: Number> Vector2D<T> {
    /// Created a vector from the given coordinates
    /// ```
    /// # use agb_fixnum::*;
    /// let v = Vector2D::new(1, 2);
    /// assert_eq!(v.x, 1);
    /// assert_eq!(v.y, 2);
    /// ```
    pub const fn new(x: T, y: T) -> Self {
        Vector2D { x, y }
    }

    /// Returns the tuple of the coordinates
    /// ```
    /// # use agb_fixnum::*;
    /// let v = Vector2D::new(1, 2);
    /// assert_eq!(v.get(), (1, 2));
    /// ```
    pub fn get(self) -> (T, T) {
        (self.x, self.y)
    }

    #[must_use]
    /// Calculates the hadamard product of two vectors
    /// ```
    /// # use agb_fixnum::*;
    /// let v1 = Vector2D::new(2, 3);
    /// let v2 = Vector2D::new(4, 5);
    ///
    /// let r = v1.hadamard(v2);
    /// assert_eq!(r, Vector2D::new(v1.x * v2.x, v1.y * v2.y));
    /// ```
    pub fn hadamard(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    #[doc(alias = "scalar_product")]
    /// Calculates the dot product / scalar product of two vectors
    /// ```
    /// use agb_fixnum::Vector2D;
    ///
    /// let v1 = Vector2D::new(3, 5);
    /// let v2 = Vector2D::new(7, 11);
    ///
    /// let dot = v1.dot(v2);
    /// assert_eq!(dot, 76);
    /// ```
    /// The dot product for vectors *A* and *B* is defined as
    /// > *A*<sub>*x*</sub> × *B*<sub>*x*</sub> + *A*<sub>*y*</sub> × *B*<sub>*y*</sub>.
    pub fn dot(self, b: Self) -> T {
        self.x * b.x + self.y * b.y
    }

    #[doc(alias = "vector_product")]
    /// Calculates the *z* component of the cross product / vector product of two
    /// vectors
    /// ```
    /// use agb_fixnum::Vector2D;
    ///
    /// let v1 = Vector2D::new(3, 5);
    /// let v2 = Vector2D::new(7, 11);
    ///
    /// let dot = v1.cross(v2);
    /// assert_eq!(dot, -2);
    /// ```
    /// The *z* component cross product for vectors *A* and *B* is defined as
    /// > *A*<sub>*x*</sub> × *B*<sub>*y*</sub> - *A*<sub>*y*</sub> × *B*<sub>*x*</sub>.
    ///
    ///
    /// Normally the cross product / vector product is itself a vector. This is
    /// in the 3D case where the cross product of two vectors is perpendicular
    /// to both vectors. The only vector perpendicular to two 2D vectors is
    /// purely in the *z* direction, hence why this method only returns that
    /// component. The *x* and *y* components are always zero.
    pub fn cross(self, b: Self) -> T {
        self.x * b.y - self.y * b.x
    }

    #[must_use]
    /// Swaps the x and y coordinate
    /// ```
    /// # use agb_fixnum::*;
    /// let v1 = Vector2D::new(2, 3);
    /// assert_eq!(v1.swap(), Vector2D::new(3, 2));
    /// ```
    pub fn swap(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }

    #[must_use]
    /// Calculates the magnitude squared, ie (x*x + y*y)
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(3.), num!(4.)).into();
    /// assert_eq!(v1.magnitude_squared(), 25.into());
    /// ```
    pub fn magnitude_squared(self) -> T {
        self.x * self.x + self.y * self.y
    }
}

impl<T: Number + Neg<Output = T>> Neg for Vector2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        (-self.x, -self.y).into()
    }
}

#[cfg(test)]
mod test {
    use crate::FixedNum;

    use super::*;

    #[test]
    fn test_vector_multiplication_and_division() {
        let a: Vector2D<i32> = (1, 2).into();
        let b = a * 5;
        let c = b / 5;
        assert_eq!(b, (5, 10).into());
        assert_eq!(a, c);
    }

    #[test]
    fn magnitude_accuracy() {
        let n: Vector2D<Num<i32, 16>> = (3, 4).into();
        assert!((n.magnitude() - 5).abs() < num!(0.1));

        let n: Vector2D<Num<i32, 8>> = (3, 4).into();
        assert!((n.magnitude() - 5).abs() < num!(0.1));
    }

    #[test]
    fn test_vector_changing() {
        let v1: Vector2D<FixedNum<8>> = Vector2D::new(1.into(), 2.into());

        let v2 = v1.trunc();
        assert_eq!(v2.get(), (1, 2));

        assert_eq!(v1 + v1, (v2 + v2).into());
    }
}
