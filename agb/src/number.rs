use core::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, BitAnd, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, Shr,
        Sub, SubAssign,
    },
};

#[macro_export]
macro_rules! num {
    ($value:literal) => {{
        $crate::number::Num::new_from_parts(agb_macros::num!($value))
    }};
}

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
    fn zero() -> Self;
    fn one() -> Self;
    fn ten() -> Self;
    fn from_as_i32(v: i32) -> Self;
}

pub trait FixedWidthSignedInteger: FixedWidthUnsignedInteger + Neg<Output = Self> {
    #[must_use]
    fn fixed_abs(self) -> Self;
}

macro_rules! fixed_width_unsigned_integer_impl {
    ($T: ty) => {
        impl FixedWidthUnsignedInteger for $T {
            fn zero() -> Self {
                0
            }
            fn one() -> Self {
                1
            }
            fn ten() -> Self {
                10
            }
            fn from_as_i32(v: i32) -> Self {
                v as $T
            }
        }
    };
}

macro_rules! fixed_width_signed_integer_impl {
    ($T: ty) => {
        impl FixedWidthSignedInteger for $T {
            fn fixed_abs(self) -> Self {
                self.abs()
            }
        }
    };
}

fixed_width_unsigned_integer_impl!(i16);
fixed_width_unsigned_integer_impl!(u16);
fixed_width_unsigned_integer_impl!(i32);
fixed_width_unsigned_integer_impl!(u32);
fixed_width_unsigned_integer_impl!(usize);

fixed_width_signed_integer_impl!(i16);
fixed_width_signed_integer_impl!(i32);

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Num<I: FixedWidthUnsignedInteger, const N: usize>(I);

pub type FixedNum<const N: usize> = Num<i32, N>;
pub type Integer = Num<i32, 0>;

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
    pub fn change_base<J: FixedWidthUnsignedInteger + From<I>, const M: usize>(self) -> Num<J, M> {
        let n: J = self.0.into();
        if N < M {
            Num(n << (M - N))
        } else {
            Num(n >> (N - M))
        }
    }

    pub fn from_raw(n: I) -> Self {
        Num(n)
    }

    pub fn to_raw(self) -> I {
        self.0
    }

    pub fn trunc(self) -> I {
        self.0 / (I::one() << N)
    }

    #[must_use]
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

    pub fn floor(self) -> I {
        self.0 >> N
    }

    pub fn frac(self) -> I {
        self.0 & ((I::one() << N) - I::one())
    }

    pub fn new(integral: I) -> Self {
        Self(integral << N)
    }

    pub fn new_from_parts(num: (i32, i32)) -> Self {
        Self(I::from_as_i32(((num.0) << N) + (num.1 >> (30 - N))))
    }
}

impl<const N: usize> Num<i32, N> {
    #[must_use]
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

#[test_case]
fn sqrt(_gba: &mut crate::Gba) {
    for x in 1..1024 {
        let n: Num<i32, 8> = Num::new(x * x);
        assert_eq!(n.sqrt(), x.into());
    }
}

#[test_case]
fn test_macro_conversion(_gba: &mut super::Gba) {
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

impl<I: FixedWidthSignedInteger, const N: usize> Num<I, N> {
    pub fn abs(self) -> Self {
        Num(self.0.fixed_abs())
    }

    /// domain of [0, 1].
    /// see https://github.com/tarcieri/micromath/blob/24584465b48ff4e87cffb709c7848664db896b4f/src/float/cos.rs#L226
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

    #[must_use]
    pub fn sin(self) -> Self {
        let one: Self = I::one().into();
        let four: I = 4.into();
        (self + one / four).cos()
    }
}

#[test_case]
fn test_numbers(_gba: &mut super::Gba) {
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

#[test_case]
fn test_division_by_one(_gba: &mut super::Gba) {
    let one: Num<i32, 8> = 1.into();

    for i in -40..40 {
        let n: Num<i32, 8> = i.into();
        assert_eq!(n / one, n);
    }
}

#[test_case]
fn test_division_and_multiplication_by_16(_gba: &mut super::Gba) {
    let sixteen: Num<i32, 8> = 16.into();

    for i in -40..40 {
        let n: Num<i32, 8> = i.into();
        let m = n / sixteen;

        assert_eq!(m * sixteen, n);
    }
}

#[test_case]
fn test_division_by_2_and_15(_gba: &mut super::Gba) {
    let two: Num<i32, 8> = 2.into();
    let fifteen: Num<i32, 8> = 15.into();
    let thirty: Num<i32, 8> = 30.into();

    for i in -128..128 {
        let n: Num<i32, 8> = i.into();

        assert_eq!(n / two / fifteen, n / thirty);
        assert_eq!(n / fifteen / two, n / thirty);
    }
}

#[test_case]
fn test_change_base(_gba: &mut super::Gba) {
    let two: Num<i32, 9> = 2.into();
    let three: Num<i32, 4> = 3.into();

    assert_eq!(two + three.change_base(), 5.into());
    assert_eq!(three + two.change_base(), 5.into());
}

#[test_case]
fn test_rem_returns_sensible_values_for_integers(_gba: &mut super::Gba) {
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

#[test_case]
fn test_rem_returns_sensible_values_for_non_integers(_gba: &mut super::Gba) {
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

#[test_case]
fn test_rem_euclid_is_always_positive_and_sensible(_gba: &mut super::Gba) {
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

impl<I: FixedWidthUnsignedInteger, const N: usize> Display for Num<I, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut integral = self.0 >> N;
        let mask: I = (I::one() << N) - I::one();

        let mut fractional = self.0 & mask;

        // Negative fix nums are awkward to print if they have non zero fractional part.
        // This is because you can think of them as `number + non negative fraction`.
        //
        // But if you think of a negative number, you'd like it to be `negative number - non negative fraction`
        // So we have to add 1 to the integral bit, and take 1 - fractional bit
        if fractional != I::zero() && integral < I::zero() {
            integral = integral + I::one();
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Vector2D<T: Number> {
    pub x: T,
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

#[test_case]
fn test_vector_multiplication_and_division(_gba: &mut super::Gba) {
    let a: Vector2D<i32> = (1, 2).into();
    let b = a * 5;
    let c = b / 5;
    assert_eq!(b, (5, 10).into());
    assert_eq!(a, c);
}

#[cfg(feature = "alloc")]
#[cfg(test)]
mod formatting_tests {
    use super::Num;
    use alloc::format;

    #[test_case]
    fn formats_whole_numbers_correctly(_gba: &mut crate::Gba) {
        let a = Num::<i32, 8>::new(-4i32);

        assert_eq!(format!("{}", a), "-4");
    }

    #[test_case]
    fn formats_fractions_correctly(_gba: &mut crate::Gba) {
        let a = Num::<i32, 8>::new(5);
        let two = Num::<i32, 8>::new(4);
        let minus_one = Num::<i32, 8>::new(-1);

        let b: Num<i32, 8> = a / two;
        let c: Num<i32, 8> = b * minus_one;

        assert_eq!(b + c, 0.into());
        assert_eq!(format!("{}", b), "1.25");
        assert_eq!(format!("{}", c), "-1.25");
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
    pub fn trunc(self) -> Vector2D<I> {
        Vector2D {
            x: self.x.trunc(),
            y: self.y.trunc(),
        }
    }

    #[must_use]
    pub fn floor(self) -> Vector2D<I> {
        Vector2D {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
}

impl<const N: usize> Vector2D<Num<i32, N>> {
    #[must_use]
    pub fn magnitude_squared(self) -> Num<i32, N> {
        self.x * self.x + self.y * self.y
    }

    #[must_use]
    pub fn manhattan_distance(self) -> Num<i32, N> {
        self.x.abs() + self.y.abs()
    }

    #[must_use]
    pub fn magnitude(self) -> Num<i32, N> {
        self.magnitude_squared().sqrt()
    }

    // calculates the magnitude of a vector using the alpha max plus beta min
    // algorithm https://en.wikipedia.org/wiki/Alpha_max_plus_beta_min_algorithm
    // this has a maximum error of less than 4% of the true magnitude, probably
    // depending on the size of your fixed point approximation
    #[must_use]
    pub fn fast_magnitude(self) -> Num<i32, N> {
        let max = core::cmp::max(self.x, self.y);
        let min = core::cmp::min(self.x, self.y);

        max * num!(0.960433870103) + min * num!(0.397824734759)
    }

    #[must_use]
    pub fn normalise(self) -> Self {
        self / self.magnitude()
    }

    #[must_use]
    pub fn fast_normalise(self) -> Self {
        self / self.fast_magnitude()
    }
}

#[test_case]
fn magnitude_accuracy(_gba: &mut crate::Gba) {
    let n: Vector2D<Num<i32, 16>> = (3, 4).into();
    assert!((n.magnitude() - 5).abs() < num!(0.1));

    let n: Vector2D<Num<i32, 8>> = (3, 4).into();
    assert!((n.magnitude() - 5).abs() < num!(0.1));
}

impl<T: Number, P: Number + Into<T>> From<(P, P)> for Vector2D<T> {
    fn from(f: (P, P)) -> Self {
        Vector2D::new(f.0.into(), f.1.into())
    }
}

impl<T: Number> Vector2D<T> {
    pub fn change_base<U: Number + From<T>>(self) -> Vector2D<U> {
        (self.x, self.y).into()
    }
}

impl<I: FixedWidthSignedInteger, const N: usize> Vector2D<Num<I, N>> {
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

#[derive(PartialEq, Eq, Clone)]
pub struct Rect<T: Number> {
    pub position: Vector2D<T>,
    pub size: Vector2D<T>,
}

impl<T: Number> Rect<T> {
    #[must_use]
    pub fn new(position: Vector2D<T>, size: Vector2D<T>) -> Self {
        Rect { position, size }
    }

    pub fn contains_point(&self, point: Vector2D<T>) -> bool {
        point.x > self.position.x
            && point.x < self.position.x + self.size.x
            && point.y > self.position.y
            && point.y < self.position.y + self.size.y
    }

    pub fn touches(&self, other: Rect<T>) -> bool {
        self.position.x < other.position.x + other.size.x
            && self.position.x + self.size.x > other.position.x
            && self.position.y < other.position.y + other.size.y
            && self.position.y + self.size.y > other.position.y
    }

    #[must_use]
    pub fn overlapping_rect(&self, other: Rect<T>) -> Self {
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

        Rect::new(top_left, bottom_right - top_left)
    }
}

impl<T: FixedWidthUnsignedInteger + core::iter::Step> Rect<T> {
    pub fn iter(self) -> impl Iterator<Item = (T, T)> {
        (self.position.x..=(self.position.x + self.size.x))
            .into_iter()
            .flat_map(move |x| {
                (self.position.y..=(self.position.y + self.size.y))
                    .into_iter()
                    .map(move |y| (x, y))
            })
    }
}

impl<T: Number> Vector2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2D { x, y }
    }

    pub fn get(self) -> (T, T) {
        (self.x, self.y)
    }

    #[must_use]
    pub fn hadamard(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    #[must_use]
    pub fn swap(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }
}

#[test_case]
fn test_vector_changing(_gba: &mut super::Gba) {
    let v1: Vector2D<FixedNum<8>> = Vector2D::new(1.into(), 2.into());

    let v2 = v1.trunc();
    assert_eq!(v2.get(), (1, 2));

    assert_eq!(v1 + v1, (v2 + v2).into());
}
