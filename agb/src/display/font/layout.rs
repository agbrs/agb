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
/// # core::include!("../../doctest_runner.rs");
/// use agb::display::font::{Layout, AlignmentKind, Font};
///
/// static FONT: Font = agb::include_font!("examples/font/pixelated.ttf", 8);
///
/// # fn test(_: agb::Gba) {
/// let mut layout = Layout::new("Hello, world!", &FONT, AlignmentKind::Left, 32, 200);
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
    tag: u16,

    max_group_width: i32,
}

impl Layout {
    #[must_use]
    /// Creates a new layout for the given text, font, and alignment. Generates
    /// [`LetterGroup`]s of width up to the `max_group_width`. The length of
    /// each line of text is given by `max_line_length`.
    pub fn new(
        text: &str,
        font: &'static Font,
        alignment: AlignmentKind,
        max_group_width: i32,
        max_line_length: i32,
    ) -> Self {
        let mut grouper = Grouper::default();
        grouper.pos.y = -font.line_height;

        Self {
            align: Align::new(alignment, max_line_length, font),
            text: text.into(),
            font,
            line: None,
            line_number: -1,
            grouper,

            palette_index: 1,
            tag: 0,
            max_group_width,
        }
    }
}

/// A collection of letters and a position for them
pub struct LetterGroup {
    tag: u16,
    str: Rc<str>,
    range: Range<usize>,
    palette_index: u8,
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

/// A set of tags currently present on the group.
///
/// Can be interrogated using the [`Self::contains`] method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tags(u16);

impl Tags {
    /// Returns true if the tag is set on this collection of tags
    #[must_use]
    pub fn contains(self, tag: Tag) -> bool {
        (self.0 >> tag.0) & 1 == 1
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
    /// # core::include!("../../doctest_runner.rs");
    /// extern crate alloc;
    /// use agb::display::font::{Font, Layout, Tag, AlignmentKind};
    /// use agb::include_font;
    /// static FONT: Font = include_font!("examples/font/pixelated.ttf", 8);
    ///
    /// # fn test(_: agb::Gba) {
    /// static MY_TAG: Tag = Tag::new(7);
    /// let text = alloc::format!("#{}!{}?", MY_TAG.set(), MY_TAG.unset());
    /// let mut layout = Layout::new(&text, &FONT, AlignmentKind::Left, 100, 100);
    /// assert!(!layout.next().unwrap().tag().contains(MY_TAG));
    /// assert!(layout.next().unwrap().tag().contains(MY_TAG));
    /// assert!(!layout.next().unwrap().tag().contains(MY_TAG));
    /// # }
    /// ```
    pub fn tag(&self) -> Tags {
        Tags(self.tag)
    }

    pub(crate) fn palette_index(&self) -> u8 {
        self.palette_index
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
    }

    /// An iterator over each pixel of the text provided as 8 packed pixels at a
    /// time. This can be used to display the text more efficiently by allowing
    /// you to render 8 pixels at a time.
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

            let y_position = font.ascent() - letter.height as i32 - letter.ymin as i32;

            let x_offset_this = x_offset;
            x_offset += kern + letter.advance_width as i32;

            let chunks_in_a_row = (letter.width / 8).into();
            let mut row_index = 0;
            let mut chunk_in_row_index = 0;

            let palette_index: u32 = self.palette_index.into();

            letter
                .data
                .iter()
                .copied()
                .map(move |x| {
                    static PX_LUT: [u16; 16] = [
                        0x0000, 0x0001, 0x0010, 0x0011, 0x0100, 0x0101, 0x0110, 0x0111, 0x1000,
                        0x1001, 0x1010, 0x1011, 0x1100, 0x1101, 0x1110, 0x1111,
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
        })
    }

    /// An iterator over each pixel of the text
    pub fn pixels(&self) -> impl Iterator<Item = Vector2D<i32>> {
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
                (0..letter.width as usize).filter_map(move |x| {
                    let rendered = letter.bit_absolute(x, y);
                    if rendered {
                        Some((x as i32 + x_offset_this, y as i32 + y_position).into())
                    } else {
                        None
                    }
                })
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

            if letter_group.width + this_letter_width > self.max_group_width {
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
            AlignmentKind::Justify,
            32,
            100,
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
            AlignmentKind::Right,
            16,
            150,
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
            AlignmentKind::Centre,
            150,
            150,
        );

        let letter_group_lines = layout.map(|lg| lg.line()).collect::<Vec<_>>();

        assert_eq!(&letter_group_lines, &[0, 1, 2, 2, 2, 2, 2, 3, 3, 3]);
    }
}
