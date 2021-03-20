use core::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        if N % 2 == 0 {
            Num((self.0 >> (N / 2)) * (rhs.0 >> (N / 2)))
        } else {
            Num((self.0 >> (1 + N / 2)) * (rhs.0 >> (N / 2)))
        }
    }
}

impl<const N: usize> MulAssign for Num<N> {
    fn mul_assign(&mut self, rhs: Self) {
        if N % 2 == 0 {
            self.0 = (self.0 >> (N / 2)) * (rhs.0 >> (N / 2))
        } else {
            self.0 = (self.0 >> (1 + N / 2)) * (rhs.0 >> (N / 2))
        }
    }
}

impl<const N: usize> Div for Num<N> {
    type Output = Self;
    fn div(self, rhs: Num<N>) -> Self::Output {
        if N % 2 == 0 {
            Num((self.0 << (N / 2)) / (rhs.0 >> (N / 2)))
        } else {
            Num((self.0 << (1 + N / 2)) / (rhs.0 >> (N / 2)))
        }
    }
}

impl<const N: usize> DivAssign for Num<N> {
    fn div_assign(&mut self, rhs: Self) {
        if N % 2 == 0 {
            self.0 = (self.0 << (N / 2)) / (rhs.0 >> (N / 2))
        } else {
            self.0 = (self.0 << (1 + N / 2)) / (rhs.0 >> (N / 2))
        }
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
