mod align;
mod layout;
mod sprite;
mod tiled;

pub use align::AlignmentKind;
pub use layout::{Layout, LetterGroup};
pub use sprite::SpriteTextRenderer;
pub use tiled::RegularBackgroundTextRenderer;

use core::fmt::{Display, Write};

/// The text renderer renders a variable width fixed size
/// bitmap font using dynamic tiles as a rendering surface.
/// For usage see the `text_render.rs` example
pub struct FontLetter {
    pub(crate) character: char,
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) data: &'static [u8],
    pub(crate) xmin: i8,
    pub(crate) ymin: i8,
    pub(crate) advance_width: u8,
    kerning_amounts: &'static [(char, i8)],
}

impl FontLetter {
    #[must_use]
    #[allow(clippy::too_many_arguments)] // only used in macro
    pub const fn new(
        character: char,
        width: u8,
        height: u8,
        data: &'static [u8],
        xmin: i8,
        ymin: i8,
        advance_width: u8,
        kerning_amounts: &'static [(char, i8)],
    ) -> Self {
        Self {
            character,
            width,
            height,
            data,
            xmin,
            ymin,
            advance_width,
            kerning_amounts,
        }
    }

    pub(crate) const fn bit_absolute(&self, x: usize, y: usize) -> bool {
        let position = x + y * self.width as usize;
        let byte = self.data[position / 8];
        let bit = position % 8;
        ((byte >> bit) & 1) != 0
    }

    pub(crate) fn kerning_amount(&self, previous_char: char) -> i32 {
        if let Ok(index) = self
            .kerning_amounts
            .binary_search_by_key(&previous_char, |kerning_data| kerning_data.0)
        {
            self.kerning_amounts[index].1 as i32
        } else {
            0
        }
    }
}

pub struct Font {
    letters: &'static [FontLetter],
    line_height: i32,
    ascent: i32,
}

impl Font {
    #[must_use]
    pub const fn new(letters: &'static [FontLetter], line_height: i32, ascent: i32) -> Self {
        Self {
            letters,
            line_height,
            ascent,
        }
    }

    pub(crate) fn letter(&self, letter: char) -> &'static FontLetter {
        let letter = self
            .letters
            .binary_search_by_key(&letter, |letter| letter.character);

        match letter {
            Ok(index) => &self.letters[index],
            Err(_) => &self.letters[0],
        }
    }

    pub(crate) fn ascent(&self) -> i32 {
        self.ascent
    }

    #[must_use]
    pub fn line_height(&self) -> i32 {
        self.line_height
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChangeColour {
    palette_index: u8,
}

impl ChangeColour {
    const UTF8_PRIVATE_USE_START: usize = 0xE000;

    #[must_use]
    /// Creates the colour changer. Colour is a palette index and must be in the range 0..16
    pub const fn new(palette_index: usize) -> Self {
        assert!(palette_index < 16, "paletted colour must be valid (0..=15)");
        Self {
            palette_index: palette_index as u8,
        }
    }

    fn try_from_char(c: char) -> Option<Self> {
        let c = c as u32 as usize;
        if (Self::UTF8_PRIVATE_USE_START..Self::UTF8_PRIVATE_USE_START + 16).contains(&c) {
            Some(Self::new(c - Self::UTF8_PRIVATE_USE_START))
        } else {
            None
        }
    }

    const fn to_char(self) -> char {
        char::from_u32(self.palette_index as u32 + Self::UTF8_PRIVATE_USE_START as u32).unwrap()
    }
}

impl Display for ChangeColour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
}
