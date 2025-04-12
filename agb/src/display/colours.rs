#![warn(missing_docs)]
use core::fmt::Debug;

use crate::{fixnum::Num, fixnum::num};

/// Represents a pixel on the GBA. This is stored as a 15 bit number as `0b0bbbbbgggggrrrrr`. You can see
/// what would happen to your true-colour value by using the [utility site](https://agbrs.dev/colour) in the agbrs.dev website.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rgb15(pub u16);

impl Rgb15 {
    /// Creates a new Rgb15 value. You should probably use `Rgb15(<your number>)` rather than this constructor,
    /// but it exists to use as a function reference when that is useful.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// A black Rgb15 value
    pub const BLACK: Rgb15 = Rgb::new(0, 0, 0).to_rgb15();
    /// A white Rgb15 value
    pub const WHITE: Rgb15 = Rgb::new(255, 255, 255).to_rgb15();
}

impl From<Rgb> for Rgb15 {
    fn from(value: Rgb) -> Self {
        value.to_rgb15()
    }
}

impl Debug for Rgb15 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let rgb = Rgb::from(*self);
        write!(f, "Rgb15({rgb:?})")
    }
}

/// Represents a full true-colour (24-bit) RGB colour. You can convert (lossily) between this and
/// [`Rgb15`] values which actually get displayed on the GBA screen.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    /// The red component
    pub r: u8,
    /// The green component
    pub g: u8,
    /// The blue component
    pub b: u8,
}

impl Rgb {
    /// Create a new Rgb value with given red, green and blue components
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create from an [`Rgb15`] value. This exists in addition to the `From` implementation because it is `const`.
    #[must_use]
    pub const fn from_rgb15(rgb15: Rgb15) -> Self {
        let rgb15 = rgb15.0;
        let r = (rgb15 & 31) << 3;
        let g = ((rgb15 >> 5) & 31) << 3;
        let b = ((rgb15 >> 10) & 31) << 3;

        Self::new(r as u8, g as u8, b as u8)
    }

    /// Create an [`Rgb15`] value. This exists in addition to the `From` implementation because it is `const`.
    #[must_use]
    pub const fn to_rgb15(self) -> Rgb15 {
        let (r, g, b) = (self.r as u16, self.g as u16, self.b as u16);
        Rgb15(((r >> 3) & 31) | (((g >> 3) & 31) << 5) | (((b >> 3) & 31) << 10))
    }

    /// Interpolate between self and another colour. This does a simple linear interpolation. Amount should be
    /// between 0 and 1 (inclusive), but could technically extend if you want to overshoot.
    #[must_use]
    pub fn interpolate(self, other: Self, amount: Num<i32, 8>) -> Self {
        let inv_amount = num!(1.) - amount;

        Self::new(
            (inv_amount * i32::from(self.r) + amount * i32::from(other.r)).floor() as u8,
            (inv_amount * i32::from(self.g) + amount * i32::from(other.g)).floor() as u8,
            (inv_amount * i32::from(self.b) + amount * i32::from(other.b)).floor() as u8,
        )
    }
}

impl From<Rgb15> for Rgb {
    fn from(value: Rgb15) -> Self {
        Self::from_rgb15(value)
    }
}

impl Debug for Rgb {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test_case]
    fn debug_print_for_rgb(_: &mut crate::Gba) {
        let debug = format!("{:?}", Rgb::new(0x55, 0xf2, 0x2b));
        assert_eq!(debug, "#55f22b");
    }

    #[test_case]
    fn debug_print_for_rgb_leading_0(_: &mut crate::Gba) {
        let debug = format!("{:?}", Rgb::new(0x05, 0x02, 0x0b));
        assert_eq!(debug, "#05020b");
    }
}
