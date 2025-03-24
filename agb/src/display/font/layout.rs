use core::ops::Range;

use alloc::rc::Rc;

use crate::fixnum::{Vector2D, vec2};

use super::{
    ChangeColour, Font, FontLetter,
    align::{Align, AlignmentKind, Line},
};

pub struct Layout {
    text: Rc<str>,
    font: &'static Font,
    align: Align,
    line: Option<Line>,
    line_number: i32,
    grouper: Grouper,

    palette_index: u8,

    max_group_width: i32,
}

impl Layout {
    #[must_use]
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

            max_group_width,
        }
    }
}

pub struct LetterGroup {
    tag: u16,
    str: Rc<str>,
    range: Range<usize>,
    palette_index: u8,
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
    pub fn text(&self) -> &str {
        &self.str[self.range.clone()]
    }

    #[must_use]
    pub fn tag(&self) -> u16 {
        self.tag
    }

    pub(crate) fn palette_index(&self) -> u8 {
        self.palette_index
    }

    pub(crate) fn font(&self) -> &Font {
        self.font
    }

    #[must_use]
    pub fn position(&self) -> Vector2D<i32> {
        self.position
    }

    #[must_use]
    pub fn line(&self) -> i32 {
        self.line
    }

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
            tag: 0,
            str: self.text.clone(),
            range: start..start,
            palette_index: self.palette_index,
            position: self.grouper.pos,
            line: self.line_number,
            font: self.font,
        };

        let mut letter_group_width = 0;

        for (char_index, char) in self.text[self.grouper.current_idx..].char_indices() {
            let char_index = char_index + start;

            // Did we finish the line?
            if char_index == line.finish_index {
                self.line = None;
                break;
            }

            if let Some(change_colour) = ChangeColour::try_from_char(char) {
                crate::println!("Colour change to {change_colour:?}");
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

            if letter_group_width + this_letter_width > self.max_group_width {
                // If we've decided that we can't fit this letter, and there currently isn't anything
                // in the letter group at all yet, then we can never fit this character. Warn the user
                // about it and skip drawing this character.
                if letter_group.range.is_empty() {
                    crate::println!("Failed to print letter '{char}' because it is too wide");
                    continue;
                }

                break;
            }

            letter_group_width += this_letter_width;
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
