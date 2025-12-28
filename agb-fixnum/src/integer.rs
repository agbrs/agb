use core::{
    fmt::{Debug, Display},
    ops::{BitAnd, Not, Shl, Shr},
};

use num_traits::Signed;
use typewit::HasTypeWitness;

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
    + HasTypeWitness<IntegerWitness<Self>>
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

#[doc(hidden)]
/// A const implementation of the trait method `from_as_i32`.
pub const fn from_as_i32<I: FixedWidthUnsignedInteger>(n: i32) -> I {
    match HasTypeWitness::WITNESS {
        IntegerWitness::U32(x) => x.to_left(n as u32),
        IntegerWitness::I16(x) => x.to_left(n as i16),
        IntegerWitness::I32(x) => x.to_left(n as i32),
        IntegerWitness::U16(x) => x.to_left(n as u16),
        IntegerWitness::I8(x) => x.to_left(n as i8),
        IntegerWitness::U8(x) => x.to_left(n as u8),
    }
}

typewit::simple_type_witness! {
    enum IntegerWitness {
        I32 = i32,
        U32 = u32,
        I16 = i16,
        U16 = u16,
        I8 = i8,
        U8 = u8,
    }
}
