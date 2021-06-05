use core::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Num<const N: usize>(i32);

pub fn change_base<const N: usize, const M: usize>(num: Num<N>) -> Num<M> {
    if N < M {
        Num(num.0 << (M - N))
    } else {
        Num(num.0 >> (N - M))
    }
}

impl<const N: usize> From<i32> for Num<N> {
    fn from(value: i32) -> Self {
        Num(value << N)
    }
}

impl<T, const N: usize> Add<T> for Num<N>
where
    T: Into<Num<N>>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Num(self.0 + rhs.into().0)
    }
}

impl<T, const N: usize> AddAssign<T> for Num<N>
where
    T: Into<Num<N>>,
{
    fn add_assign(&mut self, rhs: T) {
        self.0 = (*self + rhs.into()).0
    }
}

impl<T, const N: usize> Sub<T> for Num<N>
where
    T: Into<Num<N>>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Num(self.0 - rhs.into().0)
    }
}

impl<T, const N: usize> SubAssign<T> for Num<N>
where
    T: Into<Num<N>>,
{
    fn sub_assign(&mut self, rhs: T) {
        self.0 = (*self - rhs.into()).0
    }
}

impl<T, const N: usize> Mul<T> for Num<N>
where
    T: Into<Num<N>>,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Num((self.0 * rhs.into().0) >> N)
    }
}

impl<T, const N: usize> MulAssign<T> for Num<N>
where
    T: Into<Num<N>>,
{
    fn mul_assign(&mut self, rhs: T) {
        self.0 = (*self * rhs.into()).0
    }
}

impl<T, const N: usize> Div<T> for Num<N>
where
    T: Into<Num<N>>,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Num((self.0 << N) / rhs.into().0)
    }
}

impl<T, const N: usize> DivAssign<T> for Num<N>
where
    T: Into<Num<N>>,
{
    fn div_assign(&mut self, rhs: T) {
        self.0 = (*self / rhs.into()).0
    }
}

impl<T, const N: usize> Rem<T> for Num<N>
where
    T: Into<Num<N>>,
{
    type Output = Self;
    fn rem(self, modulus: T) -> Self::Output {
        Num(self.0 % modulus.into().0)
    }
}

impl<T, const N: usize> RemAssign<T> for Num<N>
where
    T: Into<Num<N>>,
{
    fn rem_assign(&mut self, modulus: T) {
        self.0 = (*self % modulus).0
    }
}

impl<const N: usize> Neg for Num<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Num(-self.0)
    }
}

impl<const N: usize> Num<N> {
    pub const fn max() -> Self {
        Num(i32::MAX)
    }

    pub const fn min() -> Self {
        Num(i32::MIN)
    }

    pub const fn from_raw(n: i32) -> Self {
        Num(n)
    }

    pub const fn to_raw(&self) -> i32 {
        self.0
    }

    pub const fn int(&self) -> i32 {
        let fractional_part = self.0 & ((1 << N) - 1);
        let self_as_int = self.0 >> N;

        if self_as_int < 0 && fractional_part != 0 {
            self_as_int + 1
        } else {
            self_as_int
        }
    }

    pub fn rem_euclid(&self, rhs: Self) -> Self {
        let r = *self % rhs;
        if r < 0.into() {
            if rhs < 0.into() {
                r - rhs
            } else {
                r + rhs
            }
        } else {
            r
        }
    }

    pub const fn new(integral: i32) -> Self {
        Self(integral << N)
    }
}

#[test_case]
fn test_numbers(_gba: &mut super::Gba) {
    // test addition
    let n: Num<8> = 1.into();
    assert_eq!(n + 2, 3.into(), "testing that 1 + 2 == 3");

    // test multiplication
    let n: Num<8> = 5.into();
    assert_eq!(n * 3, 15.into(), "testing that 5 * 3 == 15");

    // test division
    let n: Num<8> = 30.into();
    let p: Num<8> = 3.into();
    assert_eq!(n / 20, p / 2, "testing that 30 / 20 == 3 / 2");

    assert_ne!(n, p, "testing that 30 != 3");
}

#[test_case]
fn test_division_by_one(_gba: &mut super::Gba) {
    let one: Num<8> = 1.into();

    for i in -40..40 {
        let n: Num<8> = i.into();
        assert_eq!(n / one, n);
    }
}

#[test_case]
fn test_division_and_multiplication_by_16(_gba: &mut super::Gba) {
    let sixteen: Num<8> = 16.into();

    for i in -40..40 {
        let n: Num<8> = i.into();
        let m = n / sixteen;

        assert_eq!(m * sixteen, n);
    }
}

#[test_case]
fn test_division_by_2_and_15(_gba: &mut super::Gba) {
    let two: Num<8> = 2.into();
    let fifteen: Num<8> = 15.into();
    let thirty: Num<8> = 30.into();

    for i in -128..128 {
        let n: Num<8> = i.into();

        assert_eq!(n / two / fifteen, n / thirty);
        assert_eq!(n / fifteen / two, n / thirty);
    }
}

#[test_case]
fn test_change_base(_gba: &mut super::Gba) {
    let two: Num<9> = 2.into();
    let three: Num<4> = 3.into();

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
            let i_fixnum: Num<8> = i.into();

            assert_eq!(i_fixnum % j, i_rem_j_normally.into());
        }
    }
}

#[test_case]
fn test_rem_returns_sensible_values_for_non_integers(_gba: &mut super::Gba) {
    let one: Num<8> = 1.into();
    let third = one / 3;

    for i in -50..50 {
        for j in -50..50 {
            if j == 0 {
                continue;
            }

            // full calculation in the normal way
            let x: Num<8> = third + i;
            let y: Num<8> = j.into();

            let truncated_division: Num<8> = (x / y).int().into();

            let remainder = x - truncated_division * y;

            assert_eq!(x % y, remainder);
        }
    }
}

#[test_case]
fn test_rem_euclid_is_always_positive_and_sensible(_gba: &mut super::Gba) {
    let one: Num<8> = 1.into();
    let third = one / 3;

    for i in -50..50 {
        for j in -50..50 {
            if j == 0 {
                continue;
            }

            // full calculation in the normal way
            let x: Num<8> = third + i;
            let y: Num<8> = j.into();

            let truncated_division: Num<8> = (x / y).int().into();

            let remainder = x - truncated_division * y;

            let rem_euclid = x.rem_euclid(y);
            assert!(rem_euclid > 0.into());
        }
    }
}

impl<const N: usize> Display for Num<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let integral = self.0 >> N;
        let mask: u32 = (1 << N) - 1;

        write!(f, "{}", integral)?;

        let mut fractional = self.0 as u32 & mask;
        if fractional & mask != 0 {
            write!(f, ".")?;
        }
        while fractional & mask != 0 {
            fractional *= 10;
            write!(f, "{}", (fractional & !mask) >> N)?;
            fractional &= mask;
        }

        Ok(())
    }
}

impl<const N: usize> Debug for Num<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Num<{}>({})", N, self)
    }
}
