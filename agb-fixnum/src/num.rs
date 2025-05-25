use core::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, BitAnd, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, Shr,
        Sub, SubAssign,
    },
};

use num_traits::Signed;

mod lut {
    include!(concat!(env!("OUT_DIR"), "/lut.rs"));
}

/// Can be thought of having the signature `num!(float) -> Num<I, N>`.
/// ```
/// # use agb_fixnum::{Num, num};
/// let n: Num<i32, 8> = num!(0.75);
/// assert_eq!(n, Num::new(3) / 4, "0.75 == 3/4");
/// assert_eq!(n, num!(3. / 4.));
/// ```
#[macro_export]
macro_rules! num {
    ($value:expr) => {
        $crate::Num::new_from_parts(
            const {
                use $crate::__private::const_soft_float::soft_f64::SoftF64;

                let v = SoftF64($value as f64);
                let integer = v.trunc().to_f64();
                let fractional = v.sub(v.trunc()).to_f64() * (1_u64 << 30) as f64;

                let integer = integer as i32;
                let fractional = fractional as i32;

                (integer, fractional)
            },
        )
    };
}

/// A trait for everything required to use as the internal representation of the
/// fixed point number.
pub trait Number: Copy + PartialOrd + Ord + num_traits::Num {}
/// A trait for a signed [`Number`]
pub trait SignedNumber: Number + num_traits::Signed {}
impl<N> SignedNumber for N where N: Number + num_traits::Signed {}

impl<I: FixedWidthUnsignedInteger, const N: usize> Number for Num<I, N> {}
impl<I: FixedWidthUnsignedInteger> Number for I {}

/// A trait for integers that don't implement unary negation
pub trait FixedWidthUnsignedInteger:
    Copy
    + PartialOrd
    + Ord
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + BitAnd<Output = Self>
    + Debug
    + Display
    + num_traits::Num
    + Not<Output = Self>
    + num_traits::AsPrimitive<usize>
{
    /// Converts an i32 to it's own representation, panics on failure
    fn from_as_i32(v: i32) -> Self;
    /// Returns (a * b) >> N
    fn upcast_multiply(a: Self, b: Self, n: usize) -> Self;
}

/// Trait for an integer that includes negation
pub trait FixedWidthSignedInteger: FixedWidthUnsignedInteger + Signed {}

impl<I: FixedWidthUnsignedInteger + Signed> FixedWidthSignedInteger for I {}

macro_rules! fixed_width_unsigned_integer_impl {
    ($T: ty, $Upcast: ident) => {
        impl FixedWidthUnsignedInteger for $T {
            #[inline(always)]
            fn from_as_i32(v: i32) -> Self {
                v as $T
            }

            upcast_multiply_impl!($T, $Upcast);
        }
    };
}

macro_rules! upcast_multiply_impl {
    ($T: ty, optimised_64_bit) => {
        #[inline(always)]
        fn upcast_multiply(a: Self, b: Self, n: usize) -> Self {
            use num_traits::One;

            let mask = (Self::one() << n).wrapping_sub(1);

            let a_floor = a >> n;
            let a_frac = a & mask;

            let b_floor = b >> n;
            let b_frac = b & mask;

            (a_floor.wrapping_mul(b_floor) << n)
                .wrapping_add(
                    a_floor
                        .wrapping_mul(b_frac)
                        .wrapping_add(b_floor.wrapping_mul(a_frac)),
                )
                .wrapping_add(((a_frac as u32).wrapping_mul(b_frac as u32) >> n) as $T)
        }
    };
    ($T: ty, $Upcast: ty) => {
        #[inline(always)]
        fn upcast_multiply(a: Self, b: Self, n: usize) -> Self {
            ((<$Upcast>::from(a) * <$Upcast>::from(b)) >> n) as $T
        }
    };
}

fixed_width_unsigned_integer_impl!(i8, i32);
fixed_width_unsigned_integer_impl!(u8, u32);
fixed_width_unsigned_integer_impl!(i16, i32);
fixed_width_unsigned_integer_impl!(u16, u32);

fixed_width_unsigned_integer_impl!(i32, optimised_64_bit);
fixed_width_unsigned_integer_impl!(u32, optimised_64_bit);

/// A fixed point number represented using `I` with `N` bits of fractional precision.
///
/// These provide an alternative to floating point numbers (`f32`, `f64`) by providing
/// fractional numbers at the cost of losing the ability to store very small and very large numbers.
/// The up-side is that these are very efficient on platforms without a floating point unit
/// making arithmetic operations like `+` and `*` as fast (or almost as fast) as working
/// with integers.
///
/// # Integer type (`I`)
///
/// The `Num<I, N>` struct stores a fixed point number. The `I` represents the underlying
/// integer type for your fixed point number (e.g. `i32`, `u32`, `i16`) and `N` is the number
/// of bits you want to use for the fractional component.
///
/// We recommend using `i32` (or `u32` if you never need the number to be negative) as the
/// primitive integer type unless you have good reason not to.
///
/// # Fractional precision (`N`)
///
/// It is hard to provide general advice for how many fractional bits you will need.
/// The larger `N` is, the more precise your numbers can be, but it also reduces the maximum possible value.
/// The smallest positive number that can be represented for a given `N` will be `1 / 2^N`, and the maximum
/// will be `type::MAX / 2^N`.
///
/// <div class="warning">
/// You should ensure that `N` is less than or equal to _half_ of the number of bits in the underlying integer type.
/// So for an `i32`, you should use an `N` of _at most_ `16`.
/// </div>
///
/// # Construction
///
/// You can construct `Num` values in several ways. And which you use will depend on the circumstance.
///
/// ## 1. The [`num!`] macro (recommended)
///
/// This macro effectively has the signature `num!(value) -> Num<I, N>` where `value` is anything which can be evaluated a compile time.
/// So you can only pass constants into the [`num!`] macro, or `const` values, but not variables.
///
/// ## 2. The [`Num::new`] method (if `non-const` context)
///
/// This takes an integer value and returns a new `Num` with that _integer_ value (see the example below).
/// You can also use the `From` implementation (or `.into()`) instead of `Num::new`.
///
/// ## 3. The [`Num::from_raw`] method (not recommended)
///
/// This takes a value of type `I` which is from the underlying storage of the `Num` value.
/// You can get the raw value with [`Num::to_raw`].
/// This is mainly useful if you're storing the num values in some other mechanism (e.g. save data) and want to restore them.
/// You should prefer the other two methods for any other use-cases.
///
/// # Examples
///
/// ```rust
/// use agb_fixnum::{Num, num};
///
/// // Use the num! macro to construct using a floating point value. This will be done at compile time so
/// // the underlying platform won't ever have to do floating point calculations
/// let my_fixnum: Num<i32, 8> = num!(0.14);
/// // The new method creates with the given integer value.
/// let my_other_fixnum: Num<i32, 8> = Num::new(3);
///
/// assert_eq!(my_fixnum + my_other_fixnum, num!(3.14));
///
/// // You can add integers directly to fixnums
/// assert_eq!(my_fixnum + 3, num!(3.14));
///
/// // You can also use `.into()` or `Num::from`.
/// let my_value: Num<i32, 8> = 5.into();
/// assert_eq!(my_value, Num::from(5));
/// assert_eq!(my_value, num!(5));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Num<I: FixedWidthUnsignedInteger, const N: usize>(I);

impl<I: FixedWidthUnsignedInteger, const N: usize> num_traits::Zero for Num<I, N> {
    fn zero() -> Self {
        Self::new(I::zero())
    }

    fn is_zero(&self) -> bool {
        self.to_raw() == I::zero()
    }
}

impl<I: FixedWidthUnsignedInteger, const N: usize> num_traits::One for Num<I, N> {
    fn one() -> Self {
        Self::new(I::one())
    }
}

impl<I: FixedWidthUnsignedInteger + num_traits::Num, const N: usize> num_traits::Num for Num<I, N> {
    type FromStrRadixErr = <f64 as num_traits::Num>::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        // for some reason, if I don't have this it's an error, and if I do it is unused
        #[allow(unused_imports)]
        use num_traits::float::FloatCore;

        let v: f64 = f64::from_str_radix(str, radix)?;

        let integer = v.trunc();
        let fractional = v.fract() * (1u64 << 30) as f64;

        Ok(Self::new_from_parts((integer as i32, fractional as i32)))
    }
}

impl<I: FixedWidthUnsignedInteger + num_traits::Bounded, const N: usize> num_traits::Bounded
    for Num<I, N>
{
    fn min_value() -> Self {
        Num::from_raw(I::min_value())
    }

    fn max_value() -> Self {
        Num::from_raw(I::max_value())
    }
}

impl<I: FixedWidthUnsignedInteger + num_traits::Unsigned, const N: usize> num_traits::Unsigned
    for Num<I, N>
{
}

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
        self.0 = (*self + rhs.into()).0;
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
        self.0 = (*self - rhs.into()).0;
    }
}

impl<I, const N: usize> Mul<Num<I, N>> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
{
    type Output = Self;
    fn mul(self, rhs: Num<I, N>) -> Self::Output {
        debug_assert!(
            N * 2 <= core::mem::size_of::<I>() * 8,
            "Multiplication requires N <= number of bits / 2, but have N = {} and number of bits = {}",
            N,
            core::mem::size_of::<I>() * 8
        );

        Num(I::upcast_multiply(self.0, rhs.0, N))
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
        self.0 = (*self * rhs).0;
    }
}

impl<I, const N: usize> Div<Num<I, N>> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
{
    type Output = Self;
    fn div(self, rhs: Num<I, N>) -> Self::Output {
        debug_assert!(
            N * 2 <= core::mem::size_of::<I>() * 8,
            "Division requires N <= number of bits / 2, but have N = {} and number of bits = {}",
            N,
            core::mem::size_of::<I>() * 8
        );

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
        self.0 = (*self / rhs).0;
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
        self.0 = (*self % modulus).0;
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

    /// A bit for bit conversion from a number to a fixed num.
    /// Mainly useful for serializing numbers where you don't have access to `serde`.
    ///
    /// Get the value from the [`Num::to_raw`] method.
    ///
    /// ```
    /// use agb_fixnum::{Num, num};
    ///
    /// let pi: Num<i32, 12> = num!(3.142);
    /// assert_eq!(Num::from_raw(pi.to_raw()), pi);
    /// ```
    pub const fn from_raw(n: I) -> Self {
        Num(n)
    }

    /// The internal representation of the fixed point number
    /// Mainly useful for serializing numbers where you don't have access to `serde`.
    ///
    /// Turn this back into a `Num` with `from_raw`.
    ///
    /// ```
    /// use agb_fixnum::{Num, num};
    ///
    /// let pi: Num<i32, 12> = num!(6.283);
    /// assert_eq!(Num::from_raw(pi.to_raw()), pi);
    /// ```
    pub const fn to_raw(self) -> I {
        self.0
    }

    /// Lossily transforms an f32 into a fixed point representation.
    /// You should try not to use this and instead use the [`num!`] macro.
    #[must_use]
    pub fn from_f32(input: f32) -> Self {
        Self::from_raw(I::from_as_i32((input * (1 << N) as f32) as i32))
    }

    /// Lossily transforms an f64 into a fixed point representation.
    /// You should try not to use this and instead use the [`num!`] macro.
    #[must_use]
    pub fn from_f64(input: f64) -> Self {
        Self::from_raw(I::from_as_i32((input * f64::from(1 << N)) as i32))
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
    /// Performs the equivalent to the integer `rem_euclid`. (e.g. [`i32::rem_euclid`])
    ///
    /// So `n.rem_euclid(r)` will find the smallest _positive_ value `q` such that
    /// there is an integer `p` with the property `n = p * r + q`.
    ///
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

    /// Rounds towards the nearest integer and rounds towards positive infinity
    /// for values half way between two integers. Equivalent to `(self + num!(0.5)).floor()`.
    ///
    /// ```rust
    /// use agb_fixnum::{Num, num};
    ///
    /// fn make(a: Num<i32, 8>) -> Num<i32, 8> {
    ///     a
    /// }
    ///
    /// assert_eq!(make(num!(0.2)).round(), 0);
    /// assert_eq!(make(num!(4.5)).round(), 5);
    /// assert_eq!(make(num!(9.75)).round(), 10);
    ///
    /// assert_eq!(make(num!(-9.2)).round(), -9);
    /// assert_eq!(make(num!(-11.8)).round(), -12);
    /// assert_eq!(make(num!(-2.5)).round(), -2);
    /// ```
    pub fn round(self) -> I {
        (self + num!(0.5)).floor()
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

    /// Called by the [num!] macro in order to create a fixed point number
    #[doc(hidden)]
    #[inline(always)]
    #[must_use]
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
    ///
    /// # Panics
    /// * `N` must be even
    /// * `self` must be non-negative
    pub fn sqrt(self) -> Self {
        assert_eq!(N % 2, 0, "N must be even to be able to square root");
        assert!(self.0 >= 0, "sqrt is only valid for non-negative");
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
        Num(self.0.abs())
    }

    /// Calculates the cosine of a fixed point number with the domain of [0, 1].
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
        let n: Num<I, 8> = self.change_base();
        let n: usize = n.to_raw().as_();

        let x: i16 = lut::COS[n & 0xFF];

        let x: Num<I, 11> = Num::from_raw(I::from_as_i32(i32::from(x)));

        if N <= 8 {
            return x.change_base();
        }

        let fractional_difference_mask = (I::one() << (N - 8)) - I::one();
        let fractional_difference = self.to_raw() & fractional_difference_mask;

        if fractional_difference == I::zero() {
            return x.change_base(); // we are perfectly on the boundary
        }

        // there is a small difference, so linearly interpolate the last bit
        let next_x: i16 = lut::COS[(n + 1) & 0xFF];
        let next_x: Num<I, 11> = Num::from_raw(I::from_as_i32(i32::from(next_x)));

        let x: Self = x.change_base();
        let next_x: Self = next_x.change_base();

        // using t * next_x + (1 - t) * x doesn't have enough precision, so we use the rewritten
        // version of `t * (next_x - x) + x` and manually write out the multiplication since
        // we know that this won't overflow so we don't have to do any strange multiplication dance.
        Num::from_raw(((next_x - x) * fractional_difference).to_raw() >> (N - 8)) + x
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
        (self - num!(0.25)).cos()
    }
}

impl<I: FixedWidthSignedInteger, const N: usize> Signed for Num<I, N> {
    fn abs(&self) -> Self {
        Self::abs(*self)
    }

    fn abs_sub(&self, other: &Self) -> Self {
        Self(self.0.abs_sub(&other.0))
    }

    fn signum(&self) -> Self {
        Self(self.0.signum())
    }

    fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    fn is_negative(&self) -> bool {
        self.0.is_negative()
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
        let sign = if fractional != I::zero() && integral < I::zero() {
            integral = integral + I::one();
            fractional = (I::one() << N) - fractional;
            -1
        } else {
            1
        };

        let ten = I::from_as_i32(10);

        if let Some(precision) = f.precision() {
            let precision_multiplier = I::from_as_i32(10_i32.pow(precision as u32));

            let fractional_as_integer = fractional * precision_multiplier * ten;
            let mut fractional_as_integer = fractional_as_integer >> N;

            if fractional_as_integer % ten >= I::from_as_i32(5) {
                fractional_as_integer = fractional_as_integer + ten;
            }

            let mut fraction_to_write = fractional_as_integer / ten;

            if fraction_to_write >= precision_multiplier {
                integral = integral + I::from_as_i32(sign);
                fraction_to_write = fraction_to_write - precision_multiplier;
            }

            if sign == -1 && integral == I::zero() && fraction_to_write != I::zero() {
                write!(f, "-")?;
            }

            write!(f, "{integral}")?;

            if precision != 0 {
                write!(f, ".{fraction_to_write:#0precision$}")?;
            }
        } else {
            if sign == -1 && integral == I::zero() {
                write!(f, "-")?;
            }
            write!(f, "{integral}")?;

            if fractional != I::zero() {
                write!(f, ".")?;
            }

            while fractional & mask != I::zero() {
                fractional = fractional * ten;
                write!(f, "{}", (fractional & !mask) >> N)?;
                fractional = fractional & mask;
            }
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

#[cfg(test)]
mod test {
    extern crate alloc;

    use core::f64::consts::TAU;

    use super::*;
    use alloc::format;

    use num_traits::Num as _;

    #[test]
    fn formats_whole_numbers_correctly() {
        let a = Num::<i32, 8>::new(-4i32);

        assert_eq!(format!("{a}"), "-4");
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
        assert_eq!(format!("{b}"), "1.25");
        assert_eq!(format!("{c}"), "-1.25");
        assert_eq!(format!("{d}"), "-0.25");
    }

    mod precision {
        use super::*;

        macro_rules! num_ {
            ($n: literal) => {{
                let a: Num<i32, 20> = num!($n);
                a
            }};
        }

        macro_rules! test_precision {
            ($TestName: ident, $Number: literal, $Expected: literal) => {
                test_precision! { $TestName, $Number, $Expected, 2 }
            };
            ($TestName: ident, $Number: literal, $Expected: literal, $Digits: literal) => {
                #[test]
                fn $TestName() {
                    assert_eq!(
                        format!("{:.width$}", num_!($Number), width = $Digits),
                        $Expected
                    );
                }
            };
        }

        test_precision!(positive_down, 1.2345678, "1.23");
        test_precision!(positive_round_up, 1.237, "1.24");
        test_precision!(negative_round_down, -1.237, "-1.24");

        test_precision!(trailing_zero, 1.5, "1.50");
        test_precision!(leading_zero, 1.05, "1.05");

        test_precision!(positive_round_to_next_integer, 3.999, "4.00");
        test_precision!(negative_round_to_next_integer, -3.999, "-4.00");

        test_precision!(negative_round_to_1, -0.999, "-1.00");
        test_precision!(positive_round_to_1, 0.999, "1.00");

        test_precision!(positive_round_to_zero, 0.001, "0.00");
        test_precision!(negative_round_to_zero, -0.001, "0.00");

        test_precision!(zero_precision_negative, -0.001, "0", 0);
        test_precision!(zero_precision_positive, 0.001, "0", 0);
    }

    #[test]
    fn test_new_from_parts(){
        let n = Num::<i32, 4>::new_from_parts((2, 1 << 26));
        assert_eq!(n.to_raw(), (2 << 4) + 1);
    }

    #[test]
    fn sqrt() {
        for x in 1..1024 {
            let n: Num<i32, 8> = Num::new(x * x);
            assert_eq!(n.sqrt(), x.into());
        }
    }

    #[test]
    #[should_panic]
    fn sqrt_must_be_positive(){
        let n: Num<i32, 8> = Num::new(-1);
        n.sqrt();
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
    fn check_16_bit_precision_i32() {
        let a: Num<i32, 16> = num!(1.923);
        let b = num!(2.723);

        assert_eq!(
            a * b,
            Num::from_raw(((i64::from(a.to_raw()) * i64::from(b.to_raw())) >> 16) as i32)
        );
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
    fn test_str_radix() {
        use alloc::string::ToString;

        macro_rules! str_radix_test {
            ($val:tt) => {
                assert_eq!(
                    Num::<i32, 8>::from_str_radix(stringify!($val), 10).unwrap(),
                    num!($val)
                );
            };
            (-$val:tt) => {
                assert_eq!(
                    Num::<i32, 8>::from_str_radix(&("-".to_string() + stringify!($val)), 10)
                        .unwrap(),
                    num!(-$val)
                );
            };
        }

        str_radix_test!(0.1);
        str_radix_test!(0.100000);
        str_radix_test!(0000.1000);
        str_radix_test!(000000.100000);
        str_radix_test!(000000.1);

        str_radix_test!(138.229);
        str_radix_test!(-138.229);
        str_radix_test!(-1321.229231);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn test_all_multiplies() {
        use super::*;

        for i in 0..u32::MAX {
            let fix_num: Num<_, 7> = Num::from_raw(i);
            let upcasted = ((i as u64 * i as u64) >> 7) as u32;

            assert_eq!((fix_num * fix_num).to_raw(), upcasted);
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn test_panics_if_invalid_multiply() {
        let x: Num<i32, 18> = num!(5);
        let y: Num<i32, 18> = num!(5);

        let _ = x * y;
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn test_panics_if_invalid_division() {
        let x: Num<i32, 18> = num!(5);
        let y: Num<i32, 18> = num!(5);

        let _ = x / y;
    }

    macro_rules! cos_test {
        ($name:ident, $N:literal, $amount:expr) => {
            #[test]
            fn $name() {
                let diff: Num<i32, $N> = Num::from_raw(1);

                for i in 0.. {
                    let i = diff * i;

                    if i > 1.into() {
                        break;
                    }

                    let i_f64 = f64::from(i.to_raw()) / f64::from(1 << $N);

                    let i_cos = i.cos();
                    let i_f64cos = (i_f64 * TAU).cos();
                    let diff = f64::from(i_cos.to_raw()) / f64::from(1 << $N) - i_f64cos;

                    assert!(diff.abs() < $amount, "Difference: {} at {}", diff, i);
                }
            }
        };
    }

    cos_test!(cos_is_reasonably_close_14, 14, 0.0011);
    cos_test!(cos_is_reasonably_close_8, 8, 0.004);
    cos_test!(cos_is_reasonably_close_4, 4, 0.07);
}
