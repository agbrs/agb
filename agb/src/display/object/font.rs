use core::fmt::Write;

use agb_fixnum::{Rect, Vector2D};
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::Font;

use self::{
    preprocess::{Line, Preprocessed, PreprocessedElement},
    renderer::{Configuration, WordRender},
};

use super::{DynamicSprite, ObjectUnmanaged, PaletteVram, Size, SpriteVram};

mod preprocess;
mod renderer;

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum WhiteSpace {
    NewLine,
    Space,
}

impl WhiteSpace {
    pub(crate) fn from_char(c: char) -> Self {
        match c {
            ' ' => WhiteSpace::Space,
            '\n' => WhiteSpace::NewLine,
            _ => panic!("char not supported whitespace"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct LetterGroup {
    sprite: SpriteVram,
    // the width of the letter group
    width: u16,
    left: i16,
}

pub struct BufferedRender<'font> {
    char_render: WordRender,
    preprocessor: Preprocessed,
    buffered_chars: VecDeque<char>,
    letters: Letters,
    font: &'font Font,
}

#[derive(Debug, Default)]
struct Letters {
    letters: Vec<LetterGroup>,
    word_lengths: Vec<u8>,
    current_word_length: usize,
    number_of_groups: usize,
}

impl Write for BufferedRender<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.input_character(c);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextAlignment {
    Left,
    Right,
    Center,
}

struct TextAlignmentSettings {
    space_width: i32,
    start_x: i32,
}

impl TextAlignment {
    fn settings(
        self,
        line: &Line,
        minimum_space_width: i32,
        size: Vector2D<i32>,
    ) -> TextAlignmentSettings {
        match self {
            TextAlignment::Left => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: 0,
            },
            TextAlignment::Right => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: size.x - line.width(),
            },
            TextAlignment::Center => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: (size.x - line.width()) / 2,
            },
        }
    }
}

impl<'font> BufferedRender<'font> {
    #[must_use]
    pub fn new(font: &'font Font, sprite_size: Size, palette: PaletteVram) -> Self {
        let config = Configuration::new(sprite_size, palette);
        BufferedRender {
            char_render: WordRender::new(config),
            preprocessor: Preprocessed::new(),
            buffered_chars: VecDeque::new(),
            letters: Default::default(),
            font,
        }
    }
}

impl BufferedRender<'_> {
    fn input_character(&mut self, character: char) {
        self.preprocessor.add_character(self.font, character);
        self.buffered_chars.push_back(character);
    }

    pub fn process(&mut self) {
        let Some(c) = self.buffered_chars.pop_front() else { return };
        match c {
            ' ' | '\n' => {
                if let Some(group) = self.char_render.finalise_letter() {
                    self.letters.letters.push(group);
                    self.letters.current_word_length += 1;
                    self.letters.number_of_groups += 1;
                }
                if self.letters.current_word_length != 0 {
                    self.letters.word_lengths.push(
                        self.letters
                            .current_word_length
                            .try_into()
                            .expect("word is too big"),
                    );
                }
                self.letters.current_word_length = 0;
                self.letters.number_of_groups += 1;
            }
            letter => {
                if let Some(group) = self.char_render.render_char(self.font, letter) {
                    self.letters.letters.push(group);
                    self.letters.current_word_length += 1;
                    self.letters.number_of_groups += 1;
                }
            }
        }
    }

    #[must_use]
    pub fn layout(
        &mut self,
        area: Rect<i32>,
        alignment: TextAlignment,
        number_of_groups: usize,
        paragraph_spacing: i32,
    ) -> Vec<ObjectUnmanaged> {
        let mut objects = Vec::new();

        while !self.buffered_chars.is_empty() && self.letters.number_of_groups <= number_of_groups {
            self.process();
        }

        let minimum_space_width = self.font.letter(' ').advance_width as i32;

        let lines = self.preprocessor.lines(area.size.x, minimum_space_width);
        let mut head_position = area.position;

        let mut processed_depth = 0;
        let mut group_depth = 0;
        let mut word_depth = 0;
        let mut rendered_groups = 0;

        'outer: for line in lines {
            let settings = alignment.settings(&line, minimum_space_width, area.size);
            head_position.x += settings.start_x;

            for idx in 0..line.number_of_text_elements() {
                let element = self.preprocessor.get(processed_depth + idx);
                match element {
                    PreprocessedElement::Word(_) => {
                        for _ in 0..self
                            .letters
                            .word_lengths
                            .get(word_depth)
                            .copied()
                            .unwrap_or(u8::MAX)
                        {
                            let letter_group = &self.letters.letters[group_depth];
                            let mut object = ObjectUnmanaged::new(letter_group.sprite.clone());
                            head_position.x += letter_group.left as i32;
                            object.set_position(head_position);
                            head_position.x += letter_group.width as i32;
                            object.show();
                            objects.push(object);
                            group_depth += 1;
                            rendered_groups += 1;
                            if rendered_groups >= number_of_groups {
                                break 'outer;
                            }
                        }
                        word_depth += 1;
                    }
                    PreprocessedElement::WhiteSpace(space_type) => {
                        if space_type == WhiteSpace::NewLine {
                            head_position.y += paragraph_spacing;
                        }
                        head_position.x += settings.space_width;
                        rendered_groups += 1;
                        if rendered_groups >= number_of_groups {
                            break 'outer;
                        }
                    }
                }
            }

            processed_depth += line.number_of_text_elements();
            head_position.x = area.position.x;
            head_position.y += 9;
        }

        objects
    }
}
