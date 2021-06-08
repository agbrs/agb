use core::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, BitAnd, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, Shr,
        Sub, SubAssign,
    },
};

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
}

pub trait FixedWidthSignedInteger: FixedWidthUnsignedInteger + Neg<Output = Self> {
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

fixed_width_signed_integer_impl!(i16);
fixed_width_signed_integer_impl!(i32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Num<I: FixedWidthUnsignedInteger, const N: usize>(I);

pub type FixedNum<const N: usize> = Num<i32, N>;
pub type Integer = Num<i32, 0>;

pub fn change_base<
    I: FixedWidthUnsignedInteger,
    J: FixedWidthUnsignedInteger + Into<I>,
    const N: usize,
    const M: usize,
>(
    num: Num<J, N>,
) -> Num<I, M> {
    if N < M {
        Num(num.0.into() << (M - N))
    } else {
        Num(num.0.into() >> (N - M))
    }
}

impl<I: FixedWidthUnsignedInteger, const N: usize> From<I> for Num<I, N> {
    fn from(value: I) -> Self {
        Num(value << N)
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

impl<I, T, const N: usize> Mul<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let rhs: Self = rhs.into();

        Num(((self.floor() * rhs.floor()) << N)
            + (self.floor() * rhs.frac() + rhs.floor() * self.frac())
            + ((self.frac() * rhs.frac()) >> N))
    }
}

impl<I, T, const N: usize> MulAssign<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    fn mul_assign(&mut self, rhs: T) {
        self.0 = (*self * rhs.into()).0
    }
}

impl<I, T, const N: usize> Div<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Num((self.0 << N) / rhs.into().0)
    }
}

impl<I, T, const N: usize> DivAssign<T> for Num<I, N>
where
    I: FixedWidthUnsignedInteger,
    T: Into<Num<I, N>>,
{
    fn div_assign(&mut self, rhs: T) {
        self.0 = (*self / rhs.into()).0
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
    pub fn from_raw(n: I) -> Self {
        Num(n)
    }

    pub fn to_raw(self) -> I {
        self.0
    }

    pub fn trunc(self) -> I {
        self.0 / (I::one() << N)
    }

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
}

impl<I: FixedWidthSignedInteger, const N: usize> Num<I, N> {
    pub fn abs(self) -> Self {
        Num(self.0.fixed_abs())
    }

    /// domain of [0, 1].
    /// see https://github.com/tarcieri/micromath/blob/24584465b48ff4e87cffb709c7848664db896b4f/src/float/cos.rs#L226
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

    pub fn sin(self) -> Self {
        let one: Self = I::one().into();
        let four: I = 4.into();
        (self - one / four).cos()
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

    assert_eq!(two + change_base(three), 5.into());
    assert_eq!(three + change_base(two), 5.into());
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
        let integral = self.0 >> N;
        let mask: I = (I::one() << N) - I::one();

        write!(f, "{}", integral)?;

        let mut fractional = self.0 & mask;
        if fractional & mask != I::zero() {
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
    pub fn trunc(self) -> Vector2D<I> {
        Vector2D {
            x: self.x.trunc(),
            y: self.y.trunc(),
        }
    }
    pub fn floor(self) -> Vector2D<I> {
        Vector2D {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
}

impl<T: Number> From<(T, T)> for Vector2D<T> {
    fn from(f: (T, T)) -> Self {
        Vector2D::new(f.0, f.1)
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

pub struct Rect<T: Number> {
    pub position: Vector2D<T>,
    pub size: Vector2D<T>,
}

impl<T: Number> Vector2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2D { x, y }
    }
    pub fn get(self) -> (T, T) {
        (self.x, self.y)
    }
}

#[test_case]
fn test_vector_changing(_gba: &mut super::Gba) {
    let v1: Vector2D<FixedNum<8>> = Vector2D::new(1.into(), 2.into());

    let v2 = v1.trunc();
    assert_eq!(v2.get(), (1, 2));

    assert_eq!(v1 + v1, (v2 + v2).into());
}
