use core::fmt::Write;

use core::fmt::Display;
use core::ops::Range;

const UTF8_PRIVATE_USE_START: u32 = 0xE000;

pub(crate) const AGB_PRIVATE_USE_RANGE: Range<u32> =
    UTF8_PRIVATE_USE_START..UTF8_PRIVATE_USE_START + 48;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChangeColour {
    pub(crate) palette_index: u8,
}

impl ChangeColour {
    const RANGE: Range<u32> = UTF8_PRIVATE_USE_START..UTF8_PRIVATE_USE_START + 16;

    #[must_use]
    /// Creates the colour changer. Colour is a palette index and must be in the range 0..16
    pub const fn new(palette_index: u32) -> Self {
        assert!(palette_index < 16, "paletted colour must be valid (0..=15)");
        Self {
            palette_index: palette_index as u8,
        }
    }

    pub(crate) fn try_from_char(c: char) -> Option<Self> {
        let c = u32::from(c);
        if Self::RANGE.contains(&c) {
            Some(Self::new(c - Self::RANGE.start))
        } else {
            None
        }
    }

    pub(crate) const fn to_char(self) -> char {
        char::from_u32(self.palette_index as u32 + Self::RANGE.start).unwrap()
    }
}

impl Display for ChangeColour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SetTag(pub(crate) u8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnsetTag(pub(crate) u8);

impl SetTag {
    const RANGE: Range<u32> = UTF8_PRIVATE_USE_START + 16..UTF8_PRIVATE_USE_START + 32;

    #[must_use]
    pub const fn new(tag: u32) -> Self {
        assert!(tag < 16);
        Self(tag as u8)
    }

    pub(crate) fn try_from_char(c: char) -> Option<Self> {
        let c = u32::from(c);
        if Self::RANGE.contains(&c) {
            Some(Self::new(c - Self::RANGE.start))
        } else {
            None
        }
    }

    pub(crate) const fn to_char(self) -> char {
        char::from_u32(self.0 as u32 + Self::RANGE.start).unwrap()
    }
}

impl Display for SetTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
}

impl UnsetTag {
    const RANGE: Range<u32> = UTF8_PRIVATE_USE_START + 32..UTF8_PRIVATE_USE_START + 48;

    #[must_use]
    pub const fn new(tag: u32) -> Self {
        assert!(tag < 16);
        Self(tag as u8)
    }

    pub(crate) fn try_from_char(c: char) -> Option<Self> {
        let c = u32::from(c);
        if Self::RANGE.contains(&c) {
            Some(Self::new(c - Self::RANGE.start))
        } else {
            None
        }
    }

    pub(crate) const fn to_char(self) -> char {
        char::from_u32(self.0 as u32 + Self::RANGE.start).unwrap()
    }
}

impl Display for UnsetTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
}
