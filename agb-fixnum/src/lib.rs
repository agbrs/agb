#![no_std]
#![deny(missing_docs)]
//! Fixed point number implementation for representing non integers efficiently.
//!
//! If you are using this crate from within `agb`, you should refer to it as `agb::fixnum` rather than `agb_fixnum`.
//! This crate is updated in lockstep with `agb`.

mod num;
mod rect;
mod vec2;

#[doc(hidden)]
pub mod __private {
    pub use const_soft_float;
}

pub use num::*;
pub use rect::*;
pub use vec2::*;

#[cfg(test)]
mod tests {

    extern crate alloc;

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
    fn check_cos_accuracy() {
        let n: Num<i32, 8> = Num::new(1) / 32;
        assert_eq!(
            n.cos(),
            Num::from_f64((2. * core::f64::consts::PI / 32.).cos())
        );
    }

    #[test]
    fn check_16_bit_precision_i32() {
        let a: Num<i32, 16> = num!(1.923);
        let b = num!(2.723);

        assert_eq!(
            a * b,
            Num::from_raw(((a.to_raw() as i64 * b.to_raw() as i64) >> 16) as i32)
        )
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
        let rect: Rect<i32> = Rect::new((5_i32, 5_i32).into(), (2_i32, 2_i32).into());
        assert_eq!(
            rect.iter().collect::<alloc::vec::Vec<_>>(),
            &[
                vec2(5, 5),
                vec2(6, 5),
                vec2(7, 5),
                vec2(5, 6),
                vec2(6, 6),
                vec2(7, 6),
                vec2(5, 7),
                vec2(6, 7),
                vec2(7, 7),
            ]
        );
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
}
