//! A system for including, rendering, and displaying dynamic text.
//!
//! The agb text rendering system has support for:
//! * Unicode
//! * Variable sized letters
//! * Kerning
//! * Left, right, centre, and justified alignments
//! * Left to right text only
//!
//! It is designed such that there are two phases to the rendering:
//! * the [`Layout`] system which decides where groups of letters should be
//!   rendered.
//! * the [`ObjectTextRenderer`] and [`RegularBackgroundTextRenderer`] which
//!   take those groups of letters and display them to their relevant targets.
//!
//! These two phases interact through the [`LetterGroup`] that the [`Layout`]
//! generates.
//!
//! # Layout
//!
//! An iterator over the [`LetterGroup`]s. These allow for chunks of text to be
//! displayed in a single draw call and also enables the designer to track what
//! text is being manipulated should you want to perform any effects. For
//! example, you could play certain sounds after encountering text containing
//! certain characters.
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! # core::include!("../doctest_runner.rs");
//! use agb::display::font::{Layout, AlignmentKind, Font};
//!
//! static FONT: Font = agb::include_font!("examples/font/pixelated.ttf", 8);
//!
//! # fn test(_: agb::Gba) {
//! let mut layout = Layout::new("Hello, world!", &FONT, AlignmentKind::Left, 32, 200);
//!
//! let n = layout.next().unwrap();
//! assert_eq!(n.text(), "Hello,");
//! assert_eq!(n.position(), (0, 0).into());
//!
//! let n = layout.next().unwrap();
//! assert_eq!(n.text(), "world!");
//!
//! assert!(layout.next().is_none());
//! # }
//! ```
//!
//! # Object based target
//!
//! The [`ObjectTextRenderer`] creates objects that can be stored and displayed
//! later. A simple renderer could look like
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! # core::include!("../doctest_runner.rs");
//! extern crate alloc;
//! use alloc::vec::Vec;
//! use agb::display::{
//!     Palette16, Rgb15,
//!     font::{AlignmentKind, Font, Layout, ObjectTextRenderer},
//!     object::Size,
//! };
//!
//! static SIMPLE_PALETTE: &Palette16 = {
//!     let mut palette = [Rgb15::BLACK; 16];
//!     palette[1] = Rgb15::WHITE;
//!     &Palette16::new(palette)
//! };
//! static FONT: Font = agb::include_font!("examples/font/pixelated.ttf", 8);
//!
//! # fn test(mut gba: agb::Gba) {
//! let mut text_elements = Vec::new();
//!
//! // the actual text rendering
//!
//! let layout = Layout::new("Hello, world!", &FONT, AlignmentKind::Left, 16, 200);
//! let text_renderer = ObjectTextRenderer::new(SIMPLE_PALETTE.into(), Size::S16x16);
//!
//! for letter_group in layout {
//!     text_elements.push(text_renderer.show(&letter_group, (0, 0)));
//! }
//!
//! // display the objects in the usual means
//!
//! let mut gfx = gba.graphics.get();
//! let mut frame = gfx.frame();
//!
//! for obj in text_elements.iter() {
//!     obj.show(&mut frame);
//! }
//! # }
//! ```
//!
//! # Background tile based renderer
//!
//! The [`RegularBackgroundTextRenderer`] uses backgrounds and tiles to display
//! text. A simple renderer could look like
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! # core::include!("../doctest_runner.rs");
//! use agb::display::{
//!     Palette16, Rgb15, Priority,
//!     font::{AlignmentKind, Font, Layout, RegularBackgroundTextRenderer},
//!     tiled::{RegularBackground, VRAM_MANAGER, RegularBackgroundSize, TileFormat},
//! };
//!
//! static SIMPLE_PALETTE: &Palette16 = {
//!     let mut palette = [Rgb15::BLACK; 16];
//!     palette[1] = Rgb15::WHITE;
//!     &Palette16::new(palette)
//! };
//! static FONT: Font = agb::include_font!("examples/font/pixelated.ttf", 8);
//!
//! # fn test(mut gba: agb::Gba) {
//! VRAM_MANAGER.set_background_palette(0, SIMPLE_PALETTE);
//! let mut bg = RegularBackground::new(
//!     Priority::P0,
//!     RegularBackgroundSize::Background32x32,
//!     TileFormat::FourBpp,
//! );
//!
//! // the actual text rendering
//!
//! let layout = Layout::new("Hello, world!", &FONT, AlignmentKind::Left, 40, 200);
//! let mut text_renderer = RegularBackgroundTextRenderer::new((0, 0));
//!
//! for letter_group in layout {
//!     text_renderer.show(&mut bg, &letter_group);
//! }
//!
//! // display the background in the usual means
//!
//! let mut gfx = gba.graphics.get();
//! let mut frame = gfx.frame();
//!
//! bg.show(&mut frame);
//! # }
//! ```

#![warn(missing_docs)]
mod align;
mod layout;
mod object;
mod special;
mod tiled;

pub use align::AlignmentKind;
pub use layout::{Layout, LetterGroup};
pub use object::ObjectTextRenderer;
pub use tiled::RegularBackgroundTextRenderer;

pub use special::{ChangeColour, SetTag, UnsetTag};

/// A single letter's data required to render it.
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
    #[doc(hidden)]
    /// Unstable interface for creating a new Font, should only be used by the [`crate::include_font`] macro
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

/// A font that was imported using the [`include_font`] macro.
/// This can be used by creating a [`Layout`] that uses this font.
pub struct Font {
    letters: &'static [FontLetter],
    line_height: i32,
    ascent: i32,
}

impl Font {
    #[must_use]
    #[doc(hidden)]
    /// Unstable interface for creating a new Font, should only be used by the [`crate::include_font`] macro
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
    /// The height of a line for this font
    pub fn line_height(&self) -> i32 {
        self.line_height
    }
}
