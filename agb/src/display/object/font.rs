use core::fmt::Write;

use agb_fixnum::Vector2D;
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::{object::ObjectUnmanaged, Font};

use super::{DynamicSprite, OamIterator, PaletteVram, Size, SpriteVram};

struct LetterGroup {
    sprite: SpriteVram,
    /// x offset from the *start* of the *word*
    offset: i32,
}

struct Word {
    start_index: usize,
    end_index: usize,
    size: i32,
}

impl Word {
    fn number_of_letter_groups(&self) -> usize {
        self.end_index - self.start_index
    }
}

pub struct MetaWords {
    letters: Vec<LetterGroup>,
    words: Vec<Word>,
}

impl MetaWords {
    const fn new_empty() -> Self {
        Self {
            letters: Vec::new(),
            words: Vec::new(),
        }
    }

    fn word_iter(&self) -> impl Iterator<Item = (i32, &[LetterGroup])> {
        self.words
            .iter()
            .map(|x| (x.size, &self.letters[x.start_index..x.end_index]))
    }

    pub fn draw(&self, oam: &mut OamIterator) {
        fn inner_draw(mw: &MetaWords, oam: &mut OamIterator) -> Option<()> {
            let mut word_offset = 0;

            for (size, word) in mw.word_iter() {
                for letter_group in word.iter() {
                    let mut object = ObjectUnmanaged::new(letter_group.sprite.clone());
                    object.set_position((word_offset + letter_group.offset, 0).into());
                    object.show();
                    oam.next()?.set(&object);
                }

                word_offset += size + 10;
            }

            Some(())
        }

        let _ = inner_draw(self, oam);
    }
}

struct WorkingLetter {
    dynamic: DynamicSprite,
    // the x offset of the current letter with respect to the start of the current letter group
    x_position: i32,
    // where to render the letter from x_min to x_max
    x_offset: usize,
}

impl WorkingLetter {
    fn new(size: Size) -> Self {
        Self {
            dynamic: DynamicSprite::new(size),
            x_position: 0,
            x_offset: 0,
        }
    }
}

pub struct Configuration {
    sprite_size: Size,
    palette: PaletteVram,
}

impl Configuration {
    #[must_use]
    pub fn new(sprite_size: Size, palette: PaletteVram) -> Self {
        Self {
            sprite_size,
            palette,
        }
    }
}

pub struct WordRender<'font> {
    font: &'font Font,
    working: Working,
    finalised_metas: VecDeque<MetaWords>,
    config: Configuration,
}

struct Working {
    letter: WorkingLetter,
    meta: MetaWords,
    word_offset: i32,
}

impl<'font> Write for WordRender<'font> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

impl<'font> WordRender<'font> {
    #[must_use]
    pub fn new(font: &'font Font, config: Configuration) -> Self {
        WordRender {
            font,
            working: Working {
                letter: WorkingLetter::new(config.sprite_size),
                meta: MetaWords::new_empty(),
                word_offset: 0,
            },
            finalised_metas: VecDeque::new(),
            config,
        }
    }
}

impl WordRender<'_> {
    pub fn get_line(&mut self) -> Option<MetaWords> {
        self.finalised_metas.pop_front()
    }

    fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.finalise_line();
        } else if c == ' ' {
            self.finalise_word();
        } else {
            self.render_char(c);
        }
    }

    fn finalise_line(&mut self) {
        self.finalise_word();

        let mut final_meta = MetaWords::new_empty();
        core::mem::swap(&mut self.working.meta, &mut final_meta);
        self.finalised_metas.push_back(final_meta);
    }

    fn finalise_word(&mut self) {
        self.finalise_letter();

        let start_index = self.working.meta.words.last().map_or(0, |x| x.end_index);
        let end_index = self.working.meta.letters.len();
        let word = Word {
            start_index,
            end_index,
            size: self.working.word_offset,
        };

        self.working.meta.words.push(word);
        self.working.word_offset = 0;
    }

    fn finalise_letter(&mut self) {
        let mut final_letter = WorkingLetter::new(self.config.sprite_size);
        core::mem::swap(&mut final_letter, &mut self.working.letter);

        let sprite = final_letter.dynamic.to_vram(self.config.palette.clone());
        self.working.meta.letters.push(LetterGroup {
            sprite,
            offset: self.working.word_offset,
        });
        self.working.word_offset += final_letter.x_position;
    }

    fn render_char(&mut self, c: char) {
        let font_letter = self.font.letter(c);

        // uses more than the sprite can hold
        if self.working.letter.x_offset + font_letter.width as usize
            > self.config.sprite_size.to_width_height().0
        {
            self.finalise_letter();
        }

        self.working.letter.x_position += font_letter.xmin as i32;

        let y_position = self.font.ascent() - font_letter.height as i32 - font_letter.ymin as i32;

        for y in 0..font_letter.height as usize {
            for x in 0..font_letter.width as usize {
                let rendered = font_letter.bit_absolute(x, y);
                if rendered {
                    self.working.letter.dynamic.set_pixel(
                        x + self.working.letter.x_offset,
                        (y_position + y as i32) as usize,
                        1,
                    );
                }
            }
        }

        self.working.letter.x_position += font_letter.advance_width as i32;
        self.working.letter.x_offset += font_letter.advance_width as usize;
    }
}
