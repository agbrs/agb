use core::num::NonZeroU8;

use alloc::{collections::VecDeque, vec::Vec};

use crate::display::Font;

use super::WhiteSpace;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PreprocessedElementEncoded(u8);

impl PreprocessedElementEncoded {
    pub(crate) fn decode(self) -> PreprocessedElement {
        match self.0 {
            255 => PreprocessedElement::WhiteSpace(WhiteSpace::NewLine),
            254 => PreprocessedElement::WhiteSpace(WhiteSpace::Space),
            width => PreprocessedElement::LetterGroup { width },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]

pub(crate) enum PreprocessedElement {
    LetterGroup { width: u8 },
    WhiteSpace(WhiteSpace),
}

impl PreprocessedElement {
    fn encode(self) -> PreprocessedElementEncoded {
        PreprocessedElementEncoded(match self {
            PreprocessedElement::LetterGroup { width } => width,
            PreprocessedElement::WhiteSpace(space) => match space {
                WhiteSpace::NewLine => 255,
                WhiteSpace::Space => 254,
            },
        })
    }
}

#[derive(Default, Debug)]
pub(crate) struct Preprocessed {
    widths: VecDeque<PreprocessedElementEncoded>,
    preprocessor: Preprocessor,
}

#[derive(Debug, Default)]
struct Preprocessor {
    width_in_sprite: i32,
}

impl Preprocessor {
    fn add_character(
        &mut self,
        font: &Font,
        character: char,
        sprite_width: i32,
        widths: &mut VecDeque<PreprocessedElementEncoded>,
    ) {
        match character {
            space @ (' ' | '\n') => {
                if self.width_in_sprite != 0 {
                    widths.push_back(
                        PreprocessedElement::LetterGroup {
                            width: self.width_in_sprite as u8,
                        }
                        .encode(),
                    );
                    self.width_in_sprite = 0;
                }
                widths.push_back(
                    PreprocessedElement::WhiteSpace(WhiteSpace::from_char(space)).encode(),
                );
            }
            letter => {
                let letter = font.letter(letter);
                if self.width_in_sprite + letter.width as i32 > sprite_width {
                    widths.push_back(
                        PreprocessedElement::LetterGroup {
                            width: self.width_in_sprite as u8,
                        }
                        .encode(),
                    );
                    self.width_in_sprite = 0;
                }
                if self.width_in_sprite != 0 {
                    self.width_in_sprite += letter.xmin as i32;
                }
                self.width_in_sprite += letter.advance_width as i32;
            }
        }
    }
}

pub(crate) struct Lines<'preprocess> {
    minimum_space_width: i32,
    layout_width: i32,
    data: &'preprocess VecDeque<PreprocessedElementEncoded>,
    current_start_idx: usize,
}

pub(crate) struct Line {
    width: i32,
    number_of_text_elements: usize,
    number_of_spaces: usize,
    number_of_words: usize,
    number_of_letter_groups: usize,
}

impl Line {
    #[inline(always)]
    pub(crate) fn width(&self) -> i32 {
        self.width
    }
    #[inline(always)]
    pub(crate) fn number_of_text_elements(&self) -> usize {
        self.number_of_text_elements
    }
    #[inline(always)]
    pub(crate) fn number_of_spaces(&self) -> usize {
        self.number_of_spaces
    }
    #[inline(always)]
    pub(crate) fn number_of_words(&self) -> usize {
        self.number_of_words
    }

    #[inline(always)]
    pub(crate) fn number_of_letter_groups(&self) -> usize {
        self.number_of_letter_groups
    }
}

impl<'pre> Iterator for Lines<'pre> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_start_idx >= self.data.len() {
            return None;
        }

        let mut line_idx_length = 0;
        let mut current_line_width_pixels = 0;
        let mut spaces_after_last_word_count = 0usize;
        let mut start_of_current_word = usize::MAX;
        let mut length_of_current_word_pixels = 0;
        let mut length_of_current_word = 0;
        let mut number_of_spaces = 0;
        let mut number_of_words = 0;
        let mut number_of_letter_groups = 0;

        while let Some(next) = self.data.get(self.current_start_idx + line_idx_length) {
            match next.decode() {
                PreprocessedElement::LetterGroup { width } => {
                    if start_of_current_word == usize::MAX {
                        start_of_current_word = line_idx_length;
                    }
                    length_of_current_word_pixels += width as i32;
                    length_of_current_word += 1;
                    if current_line_width_pixels
                        + length_of_current_word_pixels
                        + spaces_after_last_word_count as i32 * self.minimum_space_width
                        >= self.layout_width
                    {
                        line_idx_length = start_of_current_word;
                        break;
                    }
                }
                PreprocessedElement::WhiteSpace(space) => {
                    if start_of_current_word != usize::MAX {
                        // flush word
                        current_line_width_pixels += length_of_current_word_pixels
                            + spaces_after_last_word_count as i32 * self.minimum_space_width;
                        number_of_spaces += spaces_after_last_word_count;
                        number_of_words += 1;
                        number_of_letter_groups += length_of_current_word;

                        // reset parser
                        length_of_current_word_pixels = 0;
                        length_of_current_word = 0;
                        start_of_current_word = usize::MAX;
                        spaces_after_last_word_count = 0;
                    }

                    match space {
                        WhiteSpace::NewLine => {
                            line_idx_length += 1;
                            break;
                        }
                        WhiteSpace::Space => {
                            spaces_after_last_word_count += 1;
                        }
                    }
                }
            };

            line_idx_length += 1;
        }

        self.current_start_idx += line_idx_length;

        Some(Line {
            width: current_line_width_pixels,
            number_of_text_elements: line_idx_length,
            number_of_spaces,
            number_of_words,
            number_of_letter_groups,
        })
    }
}

impl Preprocessed {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn add_character(&mut self, font: &Font, c: char, sprite_width: i32) {
        self.preprocessor
            .add_character(font, c, sprite_width, &mut self.widths);
    }

    pub(crate) fn pop(&mut self, line: &Line) {
        let elements = line.number_of_text_elements();
        for _ in 0..elements {
            self.widths.pop_front();
        }
    }

    pub(crate) fn lines(&self, layout_width: i32, minimum_space_width: i32) -> Lines<'_> {
        Lines {
            minimum_space_width,
            layout_width,
            data: &self.widths,
            current_start_idx: 0,
        }
    }

    pub(crate) fn lines_element(
        &self,
        layout_width: i32,
        minimum_space_width: i32,
    ) -> impl Iterator<Item = (Line, impl Iterator<Item = PreprocessedElementEncoded> + '_)> {
        let mut idx = 0;
        self.lines(layout_width, minimum_space_width).map(move |x| {
            let length = x.number_of_text_elements;

            let d = self.widths.range(idx..(idx + length)).copied();
            idx += length;
            (x, d)
        })
    }
}
