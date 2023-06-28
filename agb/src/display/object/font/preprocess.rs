use core::num::NonZeroU8;

use alloc::vec::Vec;

use crate::display::Font;

use super::WhiteSpace;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PreprocessedElement {
    Word(Word),
    WhiteSpace(WhiteSpace),
}

#[test_case]
fn check_size_of_preprocessed_element_is_correct(_: &mut crate::Gba) {
    assert_eq!(
        core::mem::size_of::<PreprocessedElement>(),
        core::mem::size_of::<Word>()
    );
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(align(4))]
pub(crate) struct Word {
    pixels: u8,
    number_of_sprites: NonZeroU8,
    index: u16,
}

impl Word {
    pub fn pixels(self) -> i32 {
        self.pixels.into()
    }
    pub fn number_of_sprites(self) -> usize {
        self.number_of_sprites.get().into()
    }
    pub fn sprite_index(self) -> usize {
        self.index.into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreprocessedElementStored(u8);

#[derive(Default, Debug)]
pub(crate) struct Preprocessed {
    widths: Vec<PreprocessedElement>,
    preprocessor: Preprocessor,
}

#[derive(Debug, Default)]
struct Preprocessor {
    current_word_width: i32,
    number_of_sprites: usize,
    width_in_sprite: i32,
    total_number_of_sprites: usize,
}

impl Preprocessor {
    fn add_character(
        &mut self,
        font: &Font,
        character: char,
        sprite_width: i32,
        widths: &mut Vec<PreprocessedElement>,
    ) {
        match character {
            space @ (' ' | '\n') => {
                if self.current_word_width != 0 {
                    self.number_of_sprites += 1;
                    self.total_number_of_sprites += 1;
                    widths.push(PreprocessedElement::Word(Word {
                        pixels: self.current_word_width.try_into().expect("word too wide"),
                        number_of_sprites: NonZeroU8::new(
                            self.number_of_sprites.try_into().expect("word too wide"),
                        )
                        .unwrap(),
                        index: (self.total_number_of_sprites - self.number_of_sprites)
                            .try_into()
                            .expect("out of range"),
                    }));
                    self.current_word_width = 0;
                    self.number_of_sprites = 0;
                    self.width_in_sprite = 0;
                }
                widths.push(PreprocessedElement::WhiteSpace(WhiteSpace::from_char(
                    space,
                )));
            }
            letter => {
                let letter = font.letter(letter);
                self.current_word_width += letter.advance_width as i32 + letter.xmin as i32;
                if self.width_in_sprite + letter.width as i32 > sprite_width {
                    self.number_of_sprites += 1;
                    self.total_number_of_sprites += 1;
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
    data: &'preprocess [PreprocessedElement],
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
            match next {
                PreprocessedElement::Word(word) => {
                    let additional_space_width =
                        additional_space_count as i32 * self.minimum_space_width;
                    let width = word.pixels();
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

    pub(crate) fn add_character(&mut self, font: &Font, c: char, sprite_width: i32) {
        self.preprocessor
            .add_character(font, c, sprite_width, &mut self.widths);
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
    ) -> impl Iterator<Item = (Line, &[PreprocessedElement])> {
        let mut idx = 0;
        self.lines(layout_width, minimum_space_width).map(move |x| {
            let length = x.number_of_text_elements;
            let d = &self.widths[idx..(idx + length)];
            idx += length;
            (x, d)
        })
    }
}
