use alloc::collections::vec_deque::VecDeque;

use crate::display::{object::DynamicSprite, FontLetter};

use super::{
    char_iterator::KerningCharIterator, configuration::CharConfigurator, Letter, TextConfig,
};

struct RenderConfig {
    palette_index: u32,
}

impl CharConfigurator for RenderConfig {
    fn switch_palette(&mut self, palette_index: u32) {
        self.palette_index = palette_index;
    }
}

pub struct LetterRender {
    iterator: KerningCharIterator,
    letters: VecDeque<Letter>,
    working_letter: DynamicSprite,
    current_x: i32,
    number_of_letters_in_current_letter: i32,
    render_config: RenderConfig,
}

impl LetterRender {
    fn add(&mut self, character: &FontLetter, kern: i32, config: &TextConfig) {
        if self.number_of_letters_in_current_letter != 0 {
            self.current_x += character.xmin as i32 + kern;
        }

        let y_position = config.font.ascent() - character.height as i32 - character.ymin as i32;

        if self.current_x + character.width as i32 > config.sprite_size.to_width_height().0 as i32 {
            self.finish_letter(config);
        }

        self.number_of_letters_in_current_letter += 1;

        for y in 0..character.height as usize {
            for x in 0..character.width as usize {
                let rendered = character.bit_absolute(x, y);
                if rendered {
                    self.working_letter.set_pixel(
                        x + self.current_x as usize,
                        (y_position + y as i32) as usize,
                        self.render_config.palette_index as usize,
                    );
                }
            }
        }

        self.current_x += character.advance_width as i32;
    }

    fn finish_letter(&mut self, config: &TextConfig) {
        let mut letter = DynamicSprite::new(config.sprite_size);
        core::mem::swap(&mut letter, &mut self.working_letter);
        let letter = letter.to_vram(config.palette.clone());
        self.letters.push_back(Letter { sprite: letter });
        self.current_x = 0;
        self.number_of_letters_in_current_letter = 0;
    }

    pub fn new(config: &TextConfig) -> Self {
        Self {
            iterator: KerningCharIterator::new(),
            letters: VecDeque::new(),
            working_letter: DynamicSprite::new(config.sprite_size),

            render_config: RenderConfig { palette_index: 1 },
            current_x: 0,
            number_of_letters_in_current_letter: 0,
        }
    }

    fn do_work_with_work_done(&mut self, text: &str, config: &TextConfig) -> bool {
        let Some((letter, kern)) = self
            .iterator
            .next(text, config.font, &mut self.render_config)
        else {
            if self.number_of_letters_in_current_letter != 0 {
                self.finish_letter(config);
            }
            return false;
        };

        if letter.character.is_ascii_whitespace() {
            self.finish_letter(config);
        } else {
            self.add(letter, kern, config);
        }

        true
    }

    pub fn do_work(&mut self, text: &str, config: &TextConfig, max_buffered_work: usize) {
        if self.letters.len() < max_buffered_work {
            self.do_work_with_work_done(text, config);
        }
    }

    pub fn next(&mut self, text: &str, config: &TextConfig) -> Option<Letter> {
        while self.letters.is_empty() {
            if !self.do_work_with_work_done(text, config) {
                break;
            }
        }

        self.letters.pop_front()
    }
}
