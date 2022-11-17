#![no_std]
#![deny(missing_docs)]
//! Fixed point number implementation for representing non integers efficiently.

use core::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    fmt::{Debug, Display},
    mem::size_of,
    ops::{
        Add, AddAssign, BitAnd, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, Shr,
        Sub, SubAssign,
    },
};

#[doc(hidden)]
/// Used internally by the [num!] macro which should be used instead.
pub use agb_macros::num as num_inner;

/// Can be thought of having the signature `num!(float) -> Num<I, N>`.
/// ```
/// # use agb_fixnum::Num;
/// # use agb_fixnum::num;
/// let n: Num<i32, 8> = num!(0.75);
/// assert_eq!(n, Num::new(3) / 4, "0.75 == 3/4");
/// ```
#[macro_export]
macro_rules! num {
    ($value:literal) => {{
        $crate::Num::new_from_parts($crate::num_inner!($value))
    }};
}

/// A trait for everything required to use as the internal representation of the
/// fixed point number.
pub trait Number:
    Sized
    + Copy
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Rem<Output = Self>
    + Div<Output = Self>
    + Mul<Output = Self>
{
}

impl<I: FixedWidthUnsignedInteger, const N: usize> Number for Num<I, N> {}
impl<I: FixedWidthUnsignedInteger> Number for I {}

/// A trait for integers that don't implement unary negation
pub trait FixedWidthUnsignedInteger:
    Sized
    + Copy
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + Rem<Output = Self>
    + Div<Output = Self>
    + Mul<Output = Self>
    + From<u8>
    + Debug
    + Display
{
    /// Returns the representation of zero
    fn zero() -> Self;
    /// Returns the representation of one
    fn one() -> Self;
    /// Returns the representation of ten
    fn ten() -> Self;
    /// Converts an i32 to it's own representation, panics on failure
    fn from_as_i32(v: i32) -> Self;
}

/// Trait for an integer that includes negation
pub trait FixedWidthSignedInteger: FixedWidthUnsignedInteger + Neg<Output = Self> {
    #[must_use]
    /// Returns the absolute value of the number
    fn fixed_abs(self) -> Self;
}

macro_rules! fixed_width_unsigned_integer_impl {
    ($T: ty) => {
        impl FixedWidthUnsignedInteger for $T {
            #[inline(always)]
            fn zero() -> Self {
                0
            }
            #[inline(always)]
            fn one() -> Self {
                1
            }
            #[inline(always)]
            fn ten() -> Self {
                10
            }
            #[inline(always)]
            fn from_as_i32(v: i32) -> Self {
                v as $T
            }
        }
    };
}

macro_rules! fixed_width_signed_integer_impl {
    ($T: ty) => {
        impl FixedWidthSignedInteger for $T {
            #[inline(always)]
            fn fixed_abs(self) -> Self {
                self.abs()
            }
        }
    };
}

fixed_width_unsigned_integer_impl!(u8);
fixed_width_unsigned_integer_impl!(i16);
fixed_width_unsigned_integer_impl!(u16);
fixed_width_unsigned_integer_impl!(i32);
fixed_width_unsigned_integer_impl!(u32);
fixed_width_unsigned_integer_impl!(usize);

fixed_width_signed_integer_impl!(i16);
fixed_width_signed_integer_impl!(i32);

/// A fixed point number represented using `I` with `N` bits of fractional precision
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Num<I: FixedWidthUnsignedInteger, const N: usize>(I);

/// An often convenient representation for the Game Boy Advance using word sized
/// internal representation for maximum efficiency
pub type FixedNum<const N: usize> = Num<i32, N>;

impl<I: FixedWidthUnsignedInteger, const N: usize> From<I> for Num<I, N> {
    fn from(value: I) -> Self {
        Num(value << N)
    }
}

impl<I, const N: usize> Default for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
{
    fn default() -> Self {
        Num(I::zero())
    }
}

impl<I, T, const N: usize> Add<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Num(self.0 + rhs.into().0)
    }
}

impl<I, T, const N: usize> AddAssign<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    fn add_assign(&mut self, rhs: T) {
        self.0 = (*self + rhs.into()).0
    }
}

impl<I, T, const N: usize> Sub<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Num(self.0 - rhs.into().0)
    }
}

impl<I, T, const N: usize> SubAssign<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    fn sub_assign(&mut self, rhs: T) {
        self.0 = (*self - rhs.into()).0
    }
}

impl<I, const N: usize> Mul<Num<I, N>> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
{
    type Output = Self;
    fn mul(self, rhs: Num<I, N>) -> Self::Output {
        Num(((self.floor() * rhs.floor()) << N)
            + (self.floor() * rhs.frac() + rhs.floor() * self.frac())
            + ((self.frac() * rhs.frac()) >> N))
    }
}

impl<I, const N: usize> Mul<I> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
{
    type Output = Self;
    fn mul(self, rhs: I) -> Self::Output {
        Num(self.0 * rhs)
    }
}

impl<I, T, const N: usize> MulAssign<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    Num<I, N>: Mul<T, Output = Num<I, N>>,
{
    fn mul_assign(&mut self, rhs: T) {
        self.0 = (*self * rhs).0
    }
}

impl<I, const N: usize> Div<Num<I, N>> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
{
    type Output = Self;
    fn div(self, rhs: Num<I, N>) -> Self::Output {
        Num((self.0 << N) / rhs.0)
    }
}

impl<I, const N: usize> Div<I> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
{
    type Output = Self;
    fn div(self, rhs: I) -> Self::Output {
        Num(self.0 / rhs)
    }
}

impl<I, T, const N: usize> DivAssign<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    Num<I, N>: Div<T, Output = Num<I, N>>,
{
    fn div_assign(&mut self, rhs: T) {
        self.0 = (*self / rhs).0
    }
}

impl<I, T, const N: usize> Rem<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    type Output = Self;
    fn rem(self, modulus: T) -> Self::Output {
        Num(self.0 % modulus.into().0)
    }
}

impl<I, T, const N: usize> RemAssign<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    fn rem_assign(&mut self, modulus: T) {
        self.0 = (*self % modulus).0
    }
}

impl<I: FixedWidthSignedInteger, const N: usize> Neg for Num<I, N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Num(-self.0)
    }
}

impl<I: FixedWidthUnsignedInteger, const N: usize> Num<I, N> {
    /// Performs the conversion between two integer types and between two different fractional precisions
    pub fn change_base<J: FixedWidthUnsignedInteger + From<I>, const M: usize>(self) -> Num<J, M> {
        let n: J = self.0.into();
        if N < M {
            Num(n << (M - N))
        } else {
            Num(n >> (N - M))
        }
    }

    /// Attempts to perform the conversion between two integer types and between
    /// two different fractional precisions
    /// ```
    /// # use agb_fixnum::*;
    /// let a: Num<i32, 8> = 1.into();
    /// let b: Option<Num<u8, 4>> = a.try_change_base();
    /// assert_eq!(b, Some(1.into()));
    ///
    /// let a: Num<i32, 8> = 18.into();
    /// let b: Option<Num<u8, 4>> = a.try_change_base();
    /// assert_eq!(b, None);
    /// ```
    pub fn try_change_base<J: FixedWidthUnsignedInteger + TryFrom<I>, const M: usize>(
        self,
    ) -> Option<Num<J, M>> {
        if size_of::<I>() > size_of::<J>() {
            // I bigger than J, perform the shift in I to preserve precision
            let n = if N < M {
                self.0 << (M - N)
            } else {
                self.0 >> (N - M)
            };

            let n = n.try_into().ok()?;

            Some(Num(n))
        } else {
            // J bigger than I, perform the shift in J to preserve precision
            let n: J = self.0.try_into().ok()?;

            let n = if N < M { n << (M - N) } else { n >> (N - M) };

            Some(Num(n))
        }
    }

    /// A bit for bit conversion from a number to a fixed num
    pub const fn from_raw(n: I) -> Self {
        Num(n)
    }

    /// The internal representation of the fixed point number
    pub fn to_raw(self) -> I {
        self.0
    }

    /// Truncates the fixed point number returning the integral part
    /// ```rust
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(5.67);
    /// assert_eq!(n.trunc(), 5);
    /// let n: Num<i32, 8> = num!(-5.67);
    /// assert_eq!(n.trunc(), -5);
    /// ```
    pub fn trunc(self) -> I {
        self.0 / (I::one() << N)
    }

    #[must_use]
    /// Performs the equivalent to the integer rem_euclid, which is modulo numbering.
    /// ```rust
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(5.67);
    /// let r: Num<i32, 8> = num!(4.);
    /// assert_eq!(n.rem_euclid(r), num!(1.67));
    ///
    /// let n: Num<i32, 8> = num!(-1.5);
    /// let r: Num<i32, 8> = num!(4.);
    /// assert_eq!(n.rem_euclid(r), num!(2.5));
    /// ```
    pub fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < I::zero().into() {
            if rhs < I::zero().into() {
                r - rhs
            } else {
                r + rhs
            }
        } else {
            r
        }
    }

    /// Performs rounding towards negative infinity
    /// ```rust
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(5.67);
    /// assert_eq!(n.floor(), 5);
    /// let n: Num<i32, 8> = num!(-5.67);
    /// assert_eq!(n.floor(), -6);
    /// ```
    pub fn floor(self) -> I {
        self.0 >> N
    }

    /// Returns the fractional component of a number as it's integer representation
    /// ```
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(5.5);
    /// assert_eq!(n.frac(), 1 << 7);
    /// ```
    pub fn frac(self) -> I {
        self.0 & ((I::one() << N) - I::one())
    }

    /// Creates an integer represented by a fixed point number
    /// ```
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = Num::new(5);
    /// assert_eq!(n.frac(), 0); // no fractional component
    /// assert_eq!(n, num!(5.)); // just equals the number 5
    /// ```
    pub fn new(integral: I) -> Self {
        Self(integral << N)
    }

    #[doc(hidden)]
    /// Called by the [num!] macro in order to create a fixed point number
    pub fn new_from_parts(num: (i32, i32)) -> Self {
        Self(I::from_as_i32(((num.0) << N) + (num.1 >> (30 - N))))
    }
}

impl<const N: usize> Num<i32, N> {
    #[must_use]
    /// Returns the square root of a number, it is calculated a digit at a time.
    /// ```
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(16.);
    /// assert_eq!(n.sqrt(), num!(4.));
    /// let n: Num<i32, 8> = num!(2.25);
    /// assert_eq!(n.sqrt(), num!(1.5));
    /// ```
    pub fn sqrt(self) -> Self {
        assert_eq!(N % 2, 0, "N must be even to be able to square root");
        assert!(self.0 >= 0, "sqrt is only valid for positive numbers");
        let mut d = 1 << 30;
        let mut x = self.0;
        let mut c = 0;

        while d > self.0 {
            d >>= 2;
        }

        while d != 0 {
            if x >= c + d {
                x -= c + d;
                c = (c >> 1) + d;
            } else {
                c >>= 1;
            }
            d >>= 2;
        }
        Self(c << (N / 2))
    }
}

impl<I: FixedWidthSignedInteger, const N: usize> Num<I, N> {
    #[must_use]
    /// Returns the absolute value of a fixed point number
    /// ```
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(5.5);
    /// assert_eq!(n.abs(), num!(5.5));
    /// let n: Num<i32, 8> = num!(-5.5);
    /// assert_eq!(n.abs(), num!(5.5));
    /// ```
    pub fn abs(self) -> Self {
        Num(self.0.fixed_abs())
    }

    /// Calculates the cosine of a fixed point number with the domain of [0, 1].
    /// Uses a [fifth order polynomial](https://github.com/tarcieri/micromath/blob/24584465b48ff4e87cffb709c7848664db896b4f/src/float/cos.rs#L226).
    /// ```
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(0.);   // 0 radians
    /// assert_eq!(n.cos(), num!(1.));
    /// let n: Num<i32, 8> = num!(0.25); // pi / 2 radians
    /// assert_eq!(n.cos(), num!(0.));
    /// let n: Num<i32, 8> = num!(0.5);  // pi radians
    /// assert_eq!(n.cos(), num!(-1.));
    /// let n: Num<i32, 8> = num!(0.75); // 3pi/2 radians
    /// assert_eq!(n.cos(), num!(0.));
    /// let n: Num<i32, 8> = num!(1.);   // 2 pi radians (whole rotation)
    /// assert_eq!(n.cos(), num!(1.));
    /// ```
    #[must_use]
    pub fn cos(self) -> Self {
        let one: Self = I::one().into();
        let mut x = self;
        let four: I = 4.into();
        let two: I = 2.into();
        let sixteen: I = 16.into();
        let nine: I = 9.into();
        let forty: I = 40.into();

        x -= one / four + (x + one / four).floor();
        x *= (x.abs() - one / two) * sixteen;
        x += x * (x.abs() - one) * (nine / forty);
        x
    }

    /// Calculates the sine of a number with domain of [0, 1].
    /// ```
    /// # use agb_fixnum::*;
    /// let n: Num<i32, 8> = num!(0.);   // 0 radians
    /// assert_eq!(n.sin(), num!(0.));
    /// let n: Num<i32, 8> = num!(0.25); // pi / 2 radians
    /// assert_eq!(n.sin(), num!(1.));
    /// let n: Num<i32, 8> = num!(0.5);  // pi radians
    /// assert_eq!(n.sin(), num!(0.));
    /// let n: Num<i32, 8> = num!(0.75); // 3pi/2 radians
    /// assert_eq!(n.sin(), num!(-1.));
    /// let n: Num<i32, 8> = num!(1.);   // 2 pi radians (whole rotation)
    /// assert_eq!(n.sin(), num!(0.));
    /// ```
    #[must_use]
    pub fn sin(self) -> Self {
        let one: Self = I::one().into();
        let four: I = 4.into();
        (self - one / four).cos()
    }
}

impl<I: FixedWidthUnsignedInteger, const N: usize> Display for Num<I, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut integral = self.0 >> N;
        let mask: I = (I::one() << N) - I::one();

        let mut fractional = self.0 & mask;

        // Negative fixnums are awkward to print if they have non zero fractional part.
        // This is because you can think of them as `number + non negative fraction`.
        //
        // But if you think of a negative number, you'd like it to be `negative number - non negative fraction`
        // So we have to add 1 to the integral bit, and take 1 - fractional bit
        if fractional != I::zero() && integral < I::zero() {
            integral = integral + I::one();
            if integral == I::zero() {
                // If the number is in the range (-1, 0), then we just bumped `integral` from -1 to 0,
                // so we need to compensate for the missing negative sign.
                write!(f, "-")?;
            }
            fractional = (I::one() << N) - fractional;
        }

        write!(f, "{}", integral)?;

        if fractional != I::zero() {
            write!(f, ".")?;
        }

        while fractional & mask != I::zero() {
            fractional = fractional * I::ten();
            write!(f, "{}", (fractional & !mask) >> N)?;
            fractional = fractional & mask;
        }

        Ok(())
    }
}

impl<I: FixedWidthUnsignedInteger, const N: usize> Debug for Num<I, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::any::type_name;

        write!(f, "Num<{}, {}>({})", type_name::<I>(), N, self)
    }
}

/// A vector of two points: (x, y) represented by integers or fixed point numbers
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Vector2D<T: Number> {
    /// The x coordinate
    pub x: T,
    /// The y coordinate
    pub y: T,
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

impl<I: FixedWidthUnsignedInteger, const N: usize> Vector2D<Num<I, N>> {
    #[must_use]
    /// Truncates the x and y coordinate, see [Num::trunc]
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
    /// Calculates the magnitude squared, ie (x*x + y*y)
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(3.), num!(4.)).into();
    /// assert_eq!(v1.magnitude_squared(), 25.into());
    /// ```
    pub fn magnitude_squared(self) -> Num<i32, N> {
        self.x * self.x + self.y * self.y
    }

    #[must_use]
    /// Calculates the manhattan distance, x.abs() + y.abs().
    /// ```
    /// # use agb_fixnum::*;
    /// let v1: Vector2D<Num<i32, 8>> = (num!(3.), num!(4.)).into();
    /// assert_eq!(v1.manhattan_distance(), 7.into());
    /// ```
    pub fn manhattan_distance(self) -> Num<i32, N> {
        self.x.abs() + self.y.abs()
    }

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// A rectangle with a position in 2d space and a 2d size
pub struct Rect<T: Number> {
    /// The position of the rectangle
    pub position: Vector2D<T>,
    /// The size of the rectangle
    pub size: Vector2D<T>,
}

impl<T: Number> Rect<T> {
    #[must_use]
    /// Creates a rectangle from it's position and size
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(2,3));
    /// assert_eq!(r.position, Vector2D::new(1,1));
    /// assert_eq!(r.size, Vector2D::new(2,3));
    /// ```
    pub fn new(position: Vector2D<T>, size: Vector2D<T>) -> Self {
        Rect { position, size }
    }

    /// Returns true if the rectangle contains the point given, note that the boundary counts as containing the rectangle.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    /// assert!(r.contains_point(Vector2D::new(1,1)));
    /// assert!(r.contains_point(Vector2D::new(2,2)));
    /// assert!(r.contains_point(Vector2D::new(3,3)));
    /// assert!(r.contains_point(Vector2D::new(4,4)));
    ///
    /// assert!(!r.contains_point(Vector2D::new(0,2)));
    /// assert!(!r.contains_point(Vector2D::new(5,2)));
    /// assert!(!r.contains_point(Vector2D::new(2,0)));
    /// assert!(!r.contains_point(Vector2D::new(2,5)));
    /// ```
    pub fn contains_point(&self, point: Vector2D<T>) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y
    }

    /// Returns true if the other rectangle touches or overlaps the first.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    ///
    /// assert!(r.touches(r.clone()));
    ///
    /// let r1 = Rect::new(Vector2D::new(2,2), Vector2D::new(3,3));
    /// assert!(r.touches(r1));
    ///
    /// let r2 = Rect::new(Vector2D::new(-10,-10), Vector2D::new(3,3));
    /// assert!(!r.touches(r2));
    /// ```
    pub fn touches(&self, other: Rect<T>) -> bool {
        self.position.x < other.position.x + other.size.x
            && self.position.x + self.size.x > other.position.x
            && self.position.y < other.position.y + other.size.y
            && self.position.y + self.size.y > other.position.y
    }

    #[must_use]
    /// Returns the rectangle that is the region that the two rectangles have in
    /// common, or [None] if they don't overlap
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    /// let r2 = Rect::new(Vector2D::new(2,2), Vector2D::new(3,3));
    ///
    /// assert_eq!(r.overlapping_rect(r2), Some(Rect::new(Vector2D::new(2,2), Vector2D::new(2,2))));
    /// ```
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    /// let r2 = Rect::new(Vector2D::new(-10,-10), Vector2D::new(3,3));
    ///
    /// assert_eq!(r.overlapping_rect(r2), None);
    /// ```
    pub fn overlapping_rect(&self, other: Rect<T>) -> Option<Self> {
        if !self.touches(other) {
            return None;
        }

        fn max<E: Number>(x: E, y: E) -> E {
            if x > y {
                x
            } else {
                y
            }
        }
        fn min<E: Number>(x: E, y: E) -> E {
            if x > y {
                y
            } else {
                x
            }
        }

        let top_left: Vector2D<T> = (
            max(self.position.x, other.position.x),
            max(self.position.y, other.position.y),
        )
            .into();
        let bottom_right: Vector2D<T> = (
            min(
                self.position.x + self.size.x,
                other.position.x + other.size.x,
            ),
            min(
                self.position.y + self.size.y,
                other.position.y + other.size.y,
            ),
        )
            .into();

        Some(Rect::new(top_left, bottom_right - top_left))
    }
}

impl<T: FixedWidthUnsignedInteger> Rect<T> {
    /// Iterate over the points in a rectangle in row major order.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(2,3));
    ///
    /// let expected_points = vec![(1,1), (2,1), (1,2), (2,2), (1,3), (2,3)];
    /// let rect_points: Vec<(i32, i32)> = r.iter().collect();
    ///
    /// assert_eq!(rect_points, expected_points);
    /// ```
    pub fn iter(self) -> impl Iterator<Item = (T, T)> {
        let mut x = self.position.x;
        let mut y = self.position.y;
        core::iter::from_fn(move || {
            if x >= self.position.x + self.size.x {
                x = self.position.x;
                y = y + T::one();
                if y >= self.position.y + self.size.y {
                    return None;
                }
            }

            let ret_x = x;
            x = x + T::one();

            Some((ret_x, y))
        })
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
    pub fn new(x: T, y: T) -> Self {
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
}

impl<T: Number + Neg<Output = T>> Neg for Vector2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        (-self.x, -self.y).into()
    }
}

#[cfg(test)]
mod tests {

    extern crate alloc;

    use super::*;
    use alloc::format;

    #[test]
    fn formats_whole_numbers_correctly() {
        let a = Num::<i32, 8>::new(-4i32);

        assert_eq!(format!("{}", a), "-4");
    }

    #[test]
    fn formats_fractions_correctly() {
        let a = Num::<i32, 8>::new(5);
        let four = Num::<i32, 8>::new(4);
        let minus_one = Num::<i32, 8>::new(-1);

        let b: Num<i32, 8> = a / four;
        let c: Num<i32, 8> = b * minus_one;
        let d: Num<i32, 8> = minus_one / four;

        assert_eq!(b + c, 0.into());
        assert_eq!(format!("{}", b), "1.25");
        assert_eq!(format!("{}", c), "-1.25");
        assert_eq!(format!("{}", d), "-0.25");
    }

    #[test]
    fn sqrt() {
        for x in 1..1024 {
            let n: Num<i32, 8> = Num::new(x * x);
            assert_eq!(n.sqrt(), x.into());
        }
    }

    #[test]
    fn test_macro_conversion() {
        fn test_positive<A: FixedWidthUnsignedInteger, const B: usize>() {
            let a: Num<A, B> = num!(1.5);
            let one = A::one() << B;
            let b = Num::from_raw(one + (one >> 1));

            assert_eq!(a, b);
        }

        fn test_negative<A: FixedWidthSignedInteger, const B: usize>() {
            let a: Num<A, B> = num!(-1.5);
            let one = A::one() << B;
            let b = Num::from_raw(one + (one >> 1));

            assert_eq!(a, -b);
        }

        fn test_base<const B: usize>() {
            test_positive::<i32, B>();
            test_positive::<u32, B>();
            test_negative::<i32, B>();

            if B < 16 {
                test_positive::<u16, B>();
                test_positive::<i16, B>();
                test_negative::<i16, B>();
            }
        }
        // some nice powers of two
        test_base::<8>();
        test_base::<4>();
        test_base::<16>();
        // not a power of two
        test_base::<10>();
        // an odd number
        test_base::<9>();
        // and a prime
        test_base::<11>();
    }

    #[test]
    fn test_numbers() {
        // test addition
        let n: Num<i32, 8> = 1.into();
        assert_eq!(n + 2, 3.into(), "testing that 1 + 2 == 3");

        // test multiplication
        let n: Num<i32, 8> = 5.into();
        assert_eq!(n * 3, 15.into(), "testing that 5 * 3 == 15");

        // test division
        let n: Num<i32, 8> = 30.into();
        let p: Num<i32, 8> = 3.into();
        assert_eq!(n / 20, p / 2, "testing that 30 / 20 == 3 / 2");

        assert_ne!(n, p, "testing that 30 != 3");
    }

    #[test]
    fn test_division_by_one() {
        let one: Num<i32, 8> = 1.into();

        for i in -40..40 {
            let n: Num<i32, 8> = i.into();
            assert_eq!(n / one, n);
        }
    }

    #[test]
    fn test_division_and_multiplication_by_16() {
        let sixteen: Num<i32, 8> = 16.into();

        for i in -40..40 {
            let n: Num<i32, 8> = i.into();
            let m = n / sixteen;

            assert_eq!(m * sixteen, n);
        }
    }

    #[test]
    fn test_division_by_2_and_15() {
        let two: Num<i32, 8> = 2.into();
        let fifteen: Num<i32, 8> = 15.into();
        let thirty: Num<i32, 8> = 30.into();

        for i in -128..128 {
            let n: Num<i32, 8> = i.into();

            assert_eq!(n / two / fifteen, n / thirty);
            assert_eq!(n / fifteen / two, n / thirty);
        }
    }

    #[test]
    fn test_change_base() {
        let two: Num<i32, 9> = 2.into();
        let three: Num<i32, 4> = 3.into();

        assert_eq!(two + three.change_base(), 5.into());
        assert_eq!(three + two.change_base(), 5.into());
    }

    #[test]
    fn test_rem_returns_sensible_values_for_integers() {
        for i in -50..50 {
            for j in -50..50 {
                if j == 0 {
                    continue;
                }

                let i_rem_j_normally = i % j;
                let i_fixnum: Num<i32, 8> = i.into();

                assert_eq!(i_fixnum % j, i_rem_j_normally.into());
            }
        }
    }

    #[test]
    fn test_rem_returns_sensible_values_for_non_integers() {
        let one: Num<i32, 8> = 1.into();
        let third = one / 3;

        for i in -50..50 {
            for j in -50..50 {
                if j == 0 {
                    continue;
                }

                // full calculation in the normal way
                let x: Num<i32, 8> = third + i;
                let y: Num<i32, 8> = j.into();

                let truncated_division: Num<i32, 8> = (x / y).trunc().into();

                let remainder = x - truncated_division * y;

                assert_eq!(x % y, remainder);
            }
        }
    }

    #[test]
    fn test_rem_euclid_is_always_positive_and_sensible() {
        let one: Num<i32, 8> = 1.into();
        let third = one / 3;

        for i in -50..50 {
            for j in -50..50 {
                if j == 0 {
                    continue;
                }

                let x: Num<i32, 8> = third + i;
                let y: Num<i32, 8> = j.into();

                let rem_euclid = x.rem_euclid(y);
                assert!(rem_euclid > 0.into());
            }
        }
    }

    #[test]
    fn test_only_frac_bits() {
        let quarter: Num<u8, 8> = num!(0.25);
        let neg_quarter: Num<i16, 15> = num!(-0.25);

        assert_eq!(quarter + quarter, num!(0.5));
        assert_eq!(neg_quarter + neg_quarter, num!(-0.5));
    }

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

    #[test]
    fn test_rect_iter() {
        let rect: Rect<i32> = Rect::new((5_i32, 5_i32).into(), (3_i32, 3_i32).into());
        assert_eq!(
            rect.iter().collect::<alloc::vec::Vec<_>>(),
            &[
                (5, 5),
                (6, 5),
                (7, 5),
                (5, 6),
                (6, 6),
                (7, 6),
                (5, 7),
                (6, 7),
                (7, 7),
            ]
        );
    }
}
