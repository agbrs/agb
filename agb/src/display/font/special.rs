use core::fmt::Write;

use core::fmt::Display;
use core::ops::Range;

const UTF8_PRIVATE_USE_START: u32 = 0xE000;

pub(crate) const AGB_PRIVATE_USE_RANGE: Range<u32> =
    UTF8_PRIVATE_USE_START..UTF8_PRIVATE_USE_START + 48;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// The way of changing the colour in a palette during text rendering. It
/// implements the `Display` trait such that it can be used in regular Rust
/// string formatting, for example `format!("Hello, {}world!",
/// ChangeColour::new(2))` would display `"world!"` using the second colour.
///
/// It is implemented using the unicode private use area. Specifically the 16
/// code points from from `0xE000` to `0xE010`.
pub struct ChangeColour {
    pub(crate) palette_index: u8,
}

impl ChangeColour {
    const RANGE: Range<u32> = UTF8_PRIVATE_USE_START..UTF8_PRIVATE_USE_START + 16;

    #[must_use]
    /// Creates the colour changer. Colour is a palette index and must be in the range 0..16
    ///
    /// # Panics
    /// Panic if the palette_index is out of range
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

    #[must_use]
    /// The char representation of the ChangeColour.
    pub const fn to_char(self) -> char {
        char::from_u32(self.palette_index as u32 + Self::RANGE.start).unwrap()
    }
}

impl Display for ChangeColour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// This sets a tag bit. Tags are a way of having user controlled data that can
/// be used for custom text effects. It is implemented using part of the unicode
/// private use area from `0xE010` to `0xE020`. See
/// [`LetterGroup::tag`][super::LetterGroup::tag] for a full example.
pub struct SetTag(pub(crate) u8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// This un-sets a tag bit. Tags are a way of having user controlled data that
/// can be used for custom text effects. It is implemented using part of the
/// unicode private use area from `0xE020` to `0xE030`. See
/// [`LetterGroup::tag`][super::LetterGroup::tag] for a full example.
pub struct UnsetTag(pub(crate) u8);

impl SetTag {
    const RANGE: Range<u32> = UTF8_PRIVATE_USE_START + 16..UTF8_PRIVATE_USE_START + 32;

    #[must_use]
    /// Creates a set tag that if used will set the given tag's bit
    ///
    /// # Panics
    /// Panics if the tag is greater than or equal to 16.
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

    #[must_use]
    /// The char representation of the SetTag.
    pub const fn to_char(self) -> char {
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
    /// Creates an UnsetTag that if used will unset the given tag's bit.
    ///
    /// # Panics
    /// Panics if the tag is greater than or equal to 16
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

    #[must_use]
    /// The char representation of the UnsetTag.
    pub(crate) const fn to_char(self) -> char {
        char::from_u32(self.0 as u32 + Self::RANGE.start).unwrap()
    }
}

impl Display for UnsetTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
}
