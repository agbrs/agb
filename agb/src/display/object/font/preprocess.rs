use alloc::vec::Vec;

use crate::display::Font;

use super::WhiteSpace;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PreprocessedElement {
    Word(u8),
    WhiteSpace(WhiteSpace),
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PreprocessedElementStored(u8);

impl core::fmt::Debug for PreprocessedElementStored {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PreprocessedElementStored")
            .field(&self.parse())
            .finish()
    }
}

impl PreprocessedElementStored {
    fn parse(self) -> PreprocessedElement {
        match self.0 {
            255 => PreprocessedElement::WhiteSpace(WhiteSpace::NewLine),
            254 => PreprocessedElement::WhiteSpace(WhiteSpace::Space),
            length => PreprocessedElement::Word(length),
        }
    }

    fn from_element(x: PreprocessedElement) -> Self {
        match x {
            PreprocessedElement::Word(length) => PreprocessedElementStored(length),
            PreprocessedElement::WhiteSpace(space) => PreprocessedElementStored(match space {
                WhiteSpace::NewLine => 255,
                WhiteSpace::Space => 254,
            }),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct Preprocessed {
    widths: Vec<PreprocessedElementStored>,
    preprocessor: Preprocessor,
}

#[derive(Debug, Default)]
struct Preprocessor {
    current_word_width: i32,
}

impl Preprocessor {
    fn add_character(
        &mut self,
        font: &Font,
        character: char,
        widths: &mut Vec<PreprocessedElementStored>,
    ) {
        match character {
            space @ (' ' | '\n') => {
                if self.current_word_width != 0 {
                    widths.push(PreprocessedElementStored::from_element(
                        PreprocessedElement::Word(
                            self.current_word_width
                                .try_into()
                                .expect("word should be small and positive"),
                        ),
                    ));
                    self.current_word_width = 0;
                }
                widths.push(PreprocessedElementStored::from_element(
                    PreprocessedElement::WhiteSpace(WhiteSpace::from_char(space)),
                ));
            }
            letter => {
                let letter = font.letter(letter);
                self.current_word_width += letter.advance_width as i32 + letter.xmin as i32;
            }
        }
    }
}

pub(crate) struct Lines<'preprocess> {
    minimum_space_width: i32,
    layout_width: i32,
    data: &'preprocess [PreprocessedElementStored],
    current_start_idx: usize,
}

pub(crate) struct Line {
    width: i32,
    number_of_text_elements: usize,
    number_of_spaces: usize,
    number_of_words: usize,
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
}

impl<'pre> Iterator for Lines<'pre> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_start_idx >= self.data.len() {
            return None;
        }

        let mut line_idx_length = 0;
        let mut current_line_width = 0;
        let mut additional_space_count = 0;
        let mut number_of_spaces = 0;
        let mut number_of_words = 0;

        while let Some(next) = self.data.get(self.current_start_idx + line_idx_length) {
            match next.parse() {
                PreprocessedElement::Word(pixels) => {
                    let additional_space_width =
                        additional_space_count as i32 * self.minimum_space_width;
                    let width = pixels as i32;
                    if width + current_line_width + additional_space_width > self.layout_width {
                        break;
                    }
                    number_of_words += 1;
                    current_line_width += width + additional_space_width;
                    number_of_spaces += additional_space_count;
                }
                PreprocessedElement::WhiteSpace(space) => match space {
                    WhiteSpace::NewLine => {
                        line_idx_length += 1;
                        break;
                    }
                    WhiteSpace::Space => {
                        additional_space_count += 1;
                    }
                },
            };

            line_idx_length += 1;
        }

        self.current_start_idx += line_idx_length;

        Some(Line {
            width: current_line_width,
            number_of_text_elements: line_idx_length,
            number_of_spaces,
            number_of_words,
        })
    }
}

impl Preprocessed {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn get(&self, idx: usize) -> PreprocessedElement {
        self.widths[idx].parse()
    }

    pub(crate) fn add_character(&mut self, font: &Font, c: char) {
        self.preprocessor.add_character(font, c, &mut self.widths);
    }

    pub(crate) fn lines(&self, layout_width: i32, minimum_space_width: i32) -> Lines<'_> {
        Lines {
            minimum_space_width,
            layout_width,
            data: &self.widths,
            current_start_idx: 0,
        }
    }

    fn lines_element(
        &self,
        layout_width: i32,
        minimum_space_width: i32,
    ) -> impl Iterator<Item = &[PreprocessedElementStored]> {
        let mut idx = 0;
        self.lines(layout_width, minimum_space_width).map(move |x| {
            let length = x.number_of_text_elements;
            let d = &self.widths[idx..(idx + length)];
            idx += length;
            d
        })
    }
}
