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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Tags are a way of having user controlled data that can be used for custom
/// text effects.
///
/// The [`Self::set`] character is implemented using part of the unicode private
/// use area from `0xE010` to `0xE020`. The [`Self::unset`] character is
/// implemented using `0xE020` to `0xE030`.
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// extern crate alloc;
/// use agb::display::font::{Font, Layout, Tag, AlignmentKind};
/// use agb::include_font;
/// static FONT: Font = include_font!("examples/font/pixelated.ttf", 8);
///
/// # #[agb::doctest]
/// # fn test(_: agb::Gba) {
/// static MY_TAG: Tag = Tag::new(7);
/// let text = alloc::format!("#{}!{}?", MY_TAG.set(), MY_TAG.unset());
/// let mut layout = Layout::new(&text, &FONT, AlignmentKind::Left, 100, 100);
/// assert!(!layout.next().unwrap().has_tag(MY_TAG));
/// assert!(layout.next().unwrap().has_tag(MY_TAG));
/// assert!(!layout.next().unwrap().has_tag(MY_TAG));
/// # }
/// ```
pub struct Tag(pub(crate) u8);

impl Tag {
    const SET_RANGE: Range<u32> = UTF8_PRIVATE_USE_START + 16..UTF8_PRIVATE_USE_START + 32;
    const UNSET_RANGE: Range<u32> = UTF8_PRIVATE_USE_START + 32..UTF8_PRIVATE_USE_START + 48;

    #[must_use]
    /// Creates a set tag that if used will set the given tag's bit
    ///
    /// # Panics
    /// Panics if the tag is greater than or equal to 16.
    pub const fn new(tag: u32) -> Self {
        assert!(tag < 16);
        Self(tag as u8)
    }

    #[must_use]
    /// A character that sets the tag bit. Uses characters from `0xE010` to `0xE020`.
    pub const fn set(&self) -> char {
        char::from_u32(self.0 as u32 + Self::SET_RANGE.start).unwrap()
    }

    #[must_use]
    /// A character that sets the tag bit. Uses characters from `0xE020` to `0xE030`.
    pub const fn unset(&self) -> char {
        char::from_u32(self.0 as u32 + Self::UNSET_RANGE.start).unwrap()
    }

    pub(crate) fn new_set(c: char) -> Option<Self> {
        let c = c as u32;
        if Self::SET_RANGE.contains(&c) {
            Some(Tag((c - Self::SET_RANGE.start) as u8))
        } else {
            None
        }
    }

    pub(crate) fn new_unset(c: char) -> Option<Self> {
        let c = c as u32;
        if Self::UNSET_RANGE.contains(&c) {
            Some(Tag((c - Self::UNSET_RANGE.start) as u8))
        } else {
            None
        }
    }
}
