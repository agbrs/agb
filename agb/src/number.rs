use core::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Num<const N: usize>(i32);

impl<const N: usize> From<i32> for Num<N> {
    fn from(value: i32) -> Self {
        Num(value << N)
    }
}

impl<const N: usize> Add for Num<N> {
    type Output = Self;
    fn add(self, rhs: Num<N>) -> Self::Output {
        Num(self.0 + rhs.0)
    }
}

impl<const N: usize> AddAssign for Num<N> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl<const N: usize> Sub for Num<N> {
    type Output = Self;
    fn sub(self, rhs: Num<N>) -> Self::Output {
        Num(self.0 - rhs.0)
    }
}

impl<const N: usize> SubAssign for Num<N> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl<const N: usize> Mul for Num<N> {
    type Output = Self;
    fn mul(self, rhs: Num<N>) -> Self::Output {
        Num((self.0 * rhs.0) >> N)
    }
}

impl<const N: usize> MulAssign for Num<N> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = (*self * rhs).0
    }
}

impl<const N: usize> Div for Num<N> {
    type Output = Self;
    fn div(self, rhs: Num<N>) -> Self::Output {
        Num((self.0 << N) / rhs.0)
    }
}

impl<const N: usize> DivAssign for Num<N> {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = (*self / rhs).0
    }
}

impl<const N: usize> Neg for Num<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Num(-self.0)
    }
}

impl<const N: usize> Num<N> {
    pub fn max() -> Self {
        Num(i32::MAX)
    }
    pub fn min() -> Self {
        Num(i32::MIN)
    }

    pub fn int(&self) -> i32 {
        self.0 >> N
    }

    pub fn new(integral: i32) -> Self {
        Self(integral << N)
    }
}

#[test_case]
fn test_numbers(_gba: &mut super::Gba) {
    // test addition
    let n: Num<8> = 1.into();
    assert_eq!(n + 2.into(), 3.into(), "testing that 1 + 2 == 3");

    // test multiplication
    let n: Num<8> = 5.into();
    assert_eq!(n * 3.into(), 15.into(), "testing that 5 * 3 == 15");

    // test division
    let n: Num<8> = 30.into();
    let p: Num<8> = 3.into();
    assert_eq!(n / 20.into(), p / 2.into(), "testing that 30 / 20 == 3 / 2");

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
