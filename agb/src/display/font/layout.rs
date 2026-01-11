use core::ops::Range;

use alloc::rc::Rc;

use crate::fixnum::{Vector2D, vec2};

use super::{
    ChangeColour, Font, FontLetter, Tag,
    align::{Align, AlignmentKind, Line},
};

/// An iterator over [`LetterGroup`]s. Incrementally lays out the given text into drawable chunks.
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// use agb::display::font::{Layout, AlignmentKind, Font, LayoutSettings};
///
/// static FONT: Font = agb::include_font!("examples/font/pixelated.ttf", 8);
///
/// # #[agb::doctest]
/// # fn test(_: agb::Gba) {
/// let mut layout = Layout::new(
///     "Hello, world!",
///     &FONT,
///     &LayoutSettings::new()
///         .with_max_line_length(200)
///         .with_max_group_width(32),
/// );
///
/// let n = layout.next().unwrap();
/// assert_eq!(n.text(), "Hello,");
/// assert_eq!(n.position(), (0, 0).into());
///
/// let n = layout.next().unwrap();
/// assert_eq!(n.text(), "world!");
///
/// assert!(layout.next().is_none());
/// # }
/// ```
pub struct Layout {
    text: Rc<str>,
    font: &'static Font,
    align: Align,
    line: Option<Line>,
    line_number: i32,
    grouper: Grouper,

    palette_index: u8,
    drop_shadow_palette_index: Option<u8>,
    tag: u16,

    max_group_width: i32,
}

/// Control how the text is laid out.
///
/// Uses a builder pattern, so you can construct it as follows:
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// use agb::display::font::{AlignmentKind, LayoutSettings};
///
/// # #[agb::doctest]
/// # fn test(_: agb::Gba) {
/// LayoutSettings::new()
///     .with_alignment(AlignmentKind::Centre)
/// # ;
/// # }
/// ```
#[derive(Clone)]
pub struct LayoutSettings {
    alignment: AlignmentKind,
    palette_index: u8,
    drop_shadow_palette_index: Option<u8>,
    max_group_width: i32,
    max_line_length: i32,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutSettings {
    /// Creates a new `LayoutSettings` with default values.
    ///
    /// Defaults:
    /// - `alignment`: [`AlignmentKind::Left`]
    /// - `palette_index`: 1
    /// - `drop_shadow_palette_index`: None (no drop shadow)
    /// - `max_group_width`: 16
    /// - `max_line_length`: 0 (unlimited)
    #[must_use]
    pub const fn new() -> Self {
        Self {
            alignment: AlignmentKind::Left,
            palette_index: 1,
            drop_shadow_palette_index: None,
            max_group_width: 16,
            max_line_length: 0,
        }
    }

    /// Sets the alignment for the text.
    ///
    /// Defaults to [`AlignmentKind::Left`].
    #[must_use]
    pub const fn with_alignment(mut self, alignment: AlignmentKind) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the palette index for the main font colour.
    ///
    /// Defaults to 1.
    #[must_use]
    pub const fn with_palette_index(mut self, palette_index: u8) -> Self {
        self.palette_index = palette_index;
        self
    }

    /// Sets the palette index for the drop-shadow colour.
    ///
    /// Defaults to None which means no drop-shadow.
    #[must_use]
    pub const fn with_drop_shadow(mut self, drop_shadow_palette_index: u8) -> Self {
        self.drop_shadow_palette_index = Some(drop_shadow_palette_index);
        self
    }

    /// Sets the maximum group width.
    ///
    /// If rendering with [`ObjectTextRenderer`](super::ObjectTextRenderer), then it is recommended that you use the width of the sprites you're
    /// using. But you can set it lower if you want the text to appear more smoothly (at the cost of using more sprites).
    ///
    /// If rendering with [`RegularBackgroundTextRenderer`](super::RegularBackgroundTextRenderer), then this should be a multiple of 8 for maximal performance.
    /// You can set this to any value and it won't change how many tiles get used. It does change performance characteristics though.
    ///
    /// Defaults to 16.
    #[must_use]
    pub const fn with_max_group_width(mut self, max_group_width: i32) -> Self {
        self.max_group_width = max_group_width;
        self
    }

    /// Sets the maximum line length. Lines will be wrapped to fit within this length.
    ///
    /// If set to 0, the line length is unlimited (lines will only break on newlines).
    ///
    /// Defaults to 0 (unlimited).
    #[must_use]
    pub const fn with_max_line_length(mut self, max_line_length: i32) -> Self {
        self.max_line_length = max_line_length;
        self
    }

    pub(super) fn alignment(&self) -> AlignmentKind {
        self.alignment
    }

    pub(super) fn palette_index(&self) -> u8 {
        self.palette_index
    }

    pub(super) fn drop_shadow_palette_index(&self) -> Option<u8> {
        self.drop_shadow_palette_index
    }

    pub(super) fn max_group_width(&self) -> i32 {
        self.max_group_width
    }

    pub(super) fn max_line_length(&self) -> i32 {
        self.max_line_length
    }
}

impl Layout {
    #[must_use]
    /// Creates a new layout for the given text, font, and alignment. Generates
    /// [`LetterGroup`]s of width up to the `max_group_width`.
    pub fn new(text: &str, font: &'static Font, settings: &LayoutSettings) -> Self {
        let mut grouper = Grouper::default();
        grouper.pos.y = -font.line_height;

        Self {
            align: Align::new(settings.alignment(), settings.max_line_length(), font),
            text: text.into(),
            font,
            line: None,
            line_number: -1,
            grouper,

            palette_index: settings.palette_index(),
            drop_shadow_palette_index: settings.drop_shadow_palette_index(),
            tag: 0,
            max_group_width: settings.max_group_width(),
        }
    }
}

/// A collection of letters and a position for them
pub struct LetterGroup {
    tag: u16,
    str: Rc<str>,
    range: Range<usize>,
    palette_index: u8,
    drop_shadow_palette_index: Option<u8>,
    width: i32,
    position: Vector2D<i32>,
    line: i32,
    font: &'static Font,
}

impl core::fmt::Debug for LetterGroup {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LetterGroup('{}' at {:?})", self.text(), self.position)
    }
}

impl LetterGroup {
    #[must_use]
    /// The underlying chunk of text this letter group contains
    pub fn text(&self) -> &str {
        &self.str[self.range.clone()]
    }

    #[must_use]
    /// Tags are user controlled data for the text. They can be used to support
    /// special text effects. Tags are set and unset using the `set` and `unset`
    /// characters provided by [`Tag`].
    ///
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// extern crate alloc;
    /// use agb::display::font::{Font, Layout, LayoutSettings, Tag};
    /// use agb::include_font;
    /// static FONT: Font = include_font!("examples/font/pixelated.ttf", 8);
    ///
    /// # #[agb::doctest]
    /// # fn test(_: agb::Gba) {
    /// static MY_TAG: Tag = Tag::new(7);
    /// let text = alloc::format!("#{}!{}?", MY_TAG.set(), MY_TAG.unset());
    /// let mut layout = Layout::new(&text, &FONT, &LayoutSettings::new().with_max_line_length(100));
    /// assert!(!layout.next().unwrap().has_tag(MY_TAG));
    /// assert!(layout.next().unwrap().has_tag(MY_TAG));
    /// assert!(!layout.next().unwrap().has_tag(MY_TAG));
    /// # }
    /// ```
    pub fn has_tag(&self, tag: Tag) -> bool {
        (self.tag >> tag.0) & 1 == 1
    }

    pub(crate) fn font(&self) -> &Font {
        self.font
    }

    #[must_use]
    /// The full precision position the letters should be drawn to
    pub fn position(&self) -> Vector2D<i32> {
        self.position
    }

    #[must_use]
    /// The line count of the text
    pub fn line(&self) -> i32 {
        self.line
    }

    #[must_use]
    /// The width and height the group occupies
    pub fn bounds(&self) -> Vector2D<i32> {
        let height = self
            .text()
            .chars()
            .map(|c| {
                let letter = self.font.letter(c);
                self.font.ascent() - letter.ymin as i32
            })
            .max()
            .unwrap_or(0);

        vec2(self.width, height)
            + if self.drop_shadow_palette_index.is_some() {
                vec2(1, 1)
            } else {
                vec2(0, 0)
            }
    }

    /// An iterator over the pixels of this letter group, yielding 8 horizontally-packed
    /// 4bpp pixels at a time.
    ///
    /// Each item is a `(position, packed_pixels)` pair where:
    /// - `position` is the top-left coordinate of the 8-pixel horizontal strip
    /// - `packed_pixels` is a `u32` containing 8 pixels in GBA 4bpp format (4 bits per pixel)
    ///
    /// This is more efficient than [`pixels`](Self::pixels) when rendering to tiles, as you can
    /// blit 8 pixels at once.
    ///
    /// If a drop shadow is configured, the shadow pixels are yielded before the main text pixels.
    pub fn pixels_packed(&self) -> impl Iterator<Item = (Vector2D<i32>, u32)> {
        let font = self.font();
        let mut previous_char = None;

        let mut x_offset = 0;

        self.text().chars().flat_map(move |c| {
            let letter = font.letter(c);
            let kern = if let Some(previous) = previous_char {
                letter.kerning_amount(previous)
            } else {
                0
            };

            previous_char = Some(c);

            let x_offset_this = x_offset;
            x_offset += kern + letter.advance_width as i32;

            let palette_index: u32 = self.palette_index.into();

            self.drop_shadow_palette_index
                .iter()
                .flat_map(move |&drop_shadow_palette_index| {
                    self.packed_pixels_for_letter(
                        letter,
                        drop_shadow_palette_index.into(),
                        x_offset_this + 1,
                        1,
                    )
                })
                .chain(self.packed_pixels_for_letter(letter, palette_index, x_offset_this, 0))
        })
    }

    // Iterates over a letter's pixel data, returning 8 horizontally-packed 4bpp pixels at a time.
    //
    // Font data is stored as 1-bit-per-pixel (on/off). This function expands it to 4bpp format
    // for the GBA. Each input byte contains 8 pixels. The lookup table (PX_LUT) expands each
    // 4-bit nibble so that each source bit becomes a 4-bit palette index (0 or 1). Multiplying
    // by palette_index then sets the actual colour. For example, input bits 0b1010 become
    // 0x1010, and if palette_index is 3, the result is 0x3030 (pixels: 0, 3, 0, 3).
    //
    // The 32-bit output packs 8 pixels: low 16 bits from the low nibble, high 16 bits from
    // the high nibble of each input byte.
    fn packed_pixels_for_letter(
        &self,
        letter: &FontLetter,
        palette_index: u32,
        x_offset: i32,
        y_offset: i32,
    ) -> impl Iterator<Item = (Vector2D<i32>, u32)> {
        let y_position = self.font.ascent() - letter.height as i32 - letter.ymin as i32 + y_offset;

        let x_offset_this = x_offset;

        let chunks_in_a_row = (letter.width / 8).into();

        let mut row_index = 0;
        let mut chunk_in_row_index = 0;

        letter
            .data
            .iter()
            .copied()
            .map(move |x| {
                static PX_LUT: [u16; 16] = [
                    0x0000, 0x0001, 0x0010, 0x0011, 0x0100, 0x0101, 0x0110, 0x0111, 0x1000, 0x1001,
                    0x1010, 0x1011, 0x1100, 0x1101, 0x1110, 0x1111,
                ];

                let px = u32::from(PX_LUT[usize::from(x & 0xF)])
                    | (u32::from(PX_LUT[usize::from(x >> 4)]) << 16);

                let px = px * palette_index;

                let unpacked = (
                    vec2(
                        chunk_in_row_index * 8 + x_offset_this,
                        y_position + row_index,
                    ),
                    px,
                );

                chunk_in_row_index += 1;
                if chunk_in_row_index >= chunks_in_a_row {
                    chunk_in_row_index = 0;
                    row_index += 1;
                }

                unpacked
            })
            .filter(|&(_, px)| px != 0)
    }

    /// An iterator over each pixel of the text, returning the location to plot a pixel and the palette index to use.
    pub fn pixels(&self) -> impl Iterator<Item = (Vector2D<i32>, u8)> {
        let font = self.font();
        let mut previous_char = None;

        let mut x_offset = 0;

        self.text().chars().flat_map(move |c| {
            let letter = font.letter(c);
            let kern = if let Some(previous) = previous_char {
                letter.kerning_amount(previous)
            } else {
                0
            };

            previous_char = Some(c);

            let y_position = font.ascent() - letter.height as i32 - letter.ymin as i32;

            let x_offset_this = x_offset;
            x_offset += kern + letter.advance_width as i32;

            (0..letter.height as usize).flat_map(move |y| {
                (0..letter.width as usize)
                    .flat_map(move |x| {
                        let rendered = letter.bit_absolute(x, y);
                        if rendered {
                            let this_position =
                                vec2(x as i32 + x_offset_this, y as i32 + y_position);

                            let drop_shadow =
                                self.drop_shadow_palette_index
                                    .map(|drop_shadow_palette_index| {
                                        (this_position + vec2(1, 1), drop_shadow_palette_index)
                                    });

                            let main = (this_position, self.palette_index);
                            [drop_shadow, Some(main)]
                        } else {
                            [None, None]
                        }
                    })
                    .flatten()
            })
        })
    }
}

impl Iterator for Layout {
    type Item = LetterGroup;

    fn next(&mut self) -> Option<Self::Item> {
        if self.grouper.current_idx == self.text.len() {
            return None;
        }

        let line = match &self.line {
            Some(line) => line,
            None => {
                let line = self.align.next(&self.text, self.font)?;
                self.line_number += 1;
                self.grouper.pos = vec2(line.left, self.grouper.pos.y + self.font.line_height);
                self.grouper.previous_char = None;
                self.grouper.current_idx = line.start_index;

                self.line = Some(line);
                self.line
                    .as_ref()
                    .expect("I set this to Some literally on the line above")
            }
        };

        let start = self.grouper.current_idx;

        let mut letter_group = LetterGroup {
            tag: self.tag,
            str: self.text.clone(),
            range: start..start,
            palette_index: self.palette_index,
            drop_shadow_palette_index: self.drop_shadow_palette_index,
            position: self.grouper.pos,
            line: self.line_number,
            font: self.font,
            width: 0,
        };

        for (char_index, char) in self.text[self.grouper.current_idx..].char_indices() {
            let char_index = char_index + start;

            // Did we finish the line?
            if char_index == line.finish_index {
                self.line = None;
                break;
            }

            if let Some(change_colour) = ChangeColour::try_from_char(char) {
                self.palette_index = change_colour.palette_index;

                if letter_group.range.is_empty() {
                    self.grouper.current_idx += char.len_utf8();
                    letter_group.range = self.grouper.current_idx..self.grouper.current_idx;
                    letter_group.palette_index = change_colour.palette_index;
                    continue;
                } else {
                    break;
                }
            }

            if let Some(set_tag) = Tag::new_set(char) {
                self.tag |= 1 << set_tag.0;
                if letter_group.range.is_empty() {
                    self.grouper.current_idx += char.len_utf8();
                    letter_group.range = self.grouper.current_idx..self.grouper.current_idx;
                    letter_group.tag = self.tag;
                    continue;
                } else {
                    break;
                }
            }

            if let Some(unset_tag) = Tag::new_unset(char) {
                self.tag &= !(1 << unset_tag.0);
                if letter_group.range.is_empty() {
                    self.grouper.current_idx += char.len_utf8();
                    letter_group.range = self.grouper.current_idx..self.grouper.current_idx;
                    letter_group.tag = self.tag;
                    continue;
                } else {
                    break;
                }
            }

            if char == ' ' {
                self.grouper.add_space(line);

                // Letter groups are always split by spaces. So if we already have something in
                // this letter group, then we are done. Otherwise, we redefine the starting position
                // for this group and restart the process.
                if letter_group.range.is_empty() {
                    letter_group.position = self.grouper.pos;
                    letter_group.range = self.grouper.current_idx..self.grouper.current_idx;

                    continue;
                } else {
                    break;
                }
            }

            let letter = self.font.letter(char);
            let kerning = self.grouper.kerning(letter);
            let letter_x = self.grouper.pos.x + kerning;

            // If this is the _first_ character in the letter group, then we need to render the
            // entire letter's width and bump the starting position of this group slightly to the
            // left. If it isn't the first character in the letter group, then adding this letter
            // to the group should not include the amount it overlaps with the previous letter.
            let this_letter_width = if letter_group.range.is_empty() {
                i32::from(letter.advance_width)
            } else {
                i32::from(letter.advance_width) + kerning
            };

            let drop_shadow_width_increase = self.drop_shadow_palette_index.is_some() as i32;

            if letter_group.width + this_letter_width + drop_shadow_width_increase
                > self.max_group_width
            {
                // If we've decided that we can't fit this letter, and there currently isn't anything
                // in the letter group at all yet, then we can never fit this character. Warn the user
                // about it and skip drawing this character.
                if letter_group.range.is_empty() {
                    crate::println!("Failed to print letter '{char}' because it is too wide");
                    continue;
                }

                break;
            }

            letter_group.width += this_letter_width;
            letter_group.position.x = letter_group.position.x.min(letter_x);
            letter_group.range.end = char_index + char.len_utf8();

            self.grouper.add_char(char, letter, kerning);
        }

        // If we decided not to put anything in the letter group at all, then this could be because
        // we just started a new line without adding anything to it. So we should recurse to try again.
        //
        // Note that this could infinitely recurse if you can never fit anything into the letter group.
        // But we check for that above.
        if letter_group.range.is_empty() {
            self.next()
        } else {
            Some(letter_group)
        }
    }
}

#[derive(Default)]
struct Grouper {
    previous_char: Option<char>,
    current_idx: usize,
    pos: Vector2D<i32>,
}

impl Grouper {
    fn kerning(&self, letter: &FontLetter) -> i32 {
        match self.previous_char {
            Some(previous_char) => letter.kerning_amount(previous_char),
            None => 0,
        }
    }

    fn add_char(&mut self, character: char, letter: &FontLetter, kerning: i32) {
        self.current_idx += character.len_utf8();
        self.previous_char = Some(character);
        self.pos.x += i32::from(letter.advance_width) + kerning;
    }

    fn add_space(&mut self, line: &Line) {
        self.current_idx += ' '.len_utf8();
        self.previous_char = None;
        self.pos.x += line.space_width;
    }
}

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use super::*;
    use crate::Gba;

    static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

    #[test_case]
    fn align_big_text_japanese(_: &mut Gba) {
        let layout = Layout::new(
            "現代社会において、情報技術の進化は目覚ましい。それは、私たちの生活様式だけでなく、思考様式にも大きな影響を与えている。例えば、スマートフォンやタブレット端末の普及により、いつでもどこでも情報にアクセスできるようになった。これにより、知識の共有やコミュニケーションが容易になり、新しい文化や価値観が生まれている。しかし、一方で、情報過多やプライバシーの問題など、新たな課題も浮上している。私たちは、これらの課題にどのように向き合い、情報技術をどのように活用していくべきだろうか。それは、私たち一人ひとりが真剣に考えるべき重要なテーマである。",
            &FONT,
            &LayoutSettings::new()
                .with_max_line_length(100)
                .with_alignment(AlignmentKind::Justify)
                .with_max_group_width(32),
        );

        for letter_group in layout {
            core::hint::black_box(letter_group);
        }
    }

    #[test_case]
    fn align_big_text_english(_: &mut Gba) {
        let layout = Layout::new(
            "This is some text which I've written as part of this example. It should go over a few lines",
            &FONT,
            &LayoutSettings::new()
                .with_max_line_length(150)
                .with_alignment(AlignmentKind::Right),
        );
        for letter_group in layout {
            core::hint::black_box(letter_group);
        }
    }

    #[test_case]
    fn tracks_line(_: &mut Gba) {
        let layout = Layout::new(
            "Hello\nWorld\nSome text that should break over multiple lines",
            &FONT,
            &LayoutSettings::new()
                .with_max_line_length(150)
                .with_alignment(AlignmentKind::Centre)
                .with_max_group_width(150),
        );

        let letter_group_lines = layout.map(|lg| lg.line()).collect::<Vec<_>>();

        assert_eq!(&letter_group_lines, &[0, 1, 2, 2, 2, 2, 2, 3, 3, 3]);
    }
}
