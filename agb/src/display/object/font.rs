use core::fmt::Write;

use agb_fixnum::Vector2D;
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::{object::ObjectUnmanaged, Font};

use super::{DynamicSprite, OamIterator, PaletteVram, Size, SpriteVram};

struct LetterGroup {
    sprite: SpriteVram,
    // the width of the letter group
    width: u16,
    left: i16,
}

enum WhiteSpace {
    Space,
    NewLine,
}

enum TextElement {
    LetterGroup(LetterGroup),
    WhiteSpace(WhiteSpace),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn check_size_of_text_element_is_expected(_: &mut crate::Gba) {
        assert_eq!(
            core::mem::size_of::<TextElement>(),
            core::mem::size_of::<LetterGroup>()
        );
    }
}

pub struct TextBlock {
    elements: Vec<TextElement>,
    cache: CachedRender,
}

pub struct CachedRender {
    objects: Vec<ObjectUnmanaged>,
    up_to: usize,
    head_position: Vector2D<i32>,
    origin: Vector2D<i32>,
}

impl TextBlock {
    fn reset_cache(&mut self, position: Vector2D<i32>) {
        self.cache.objects.clear();
        self.cache.up_to = 0;
        self.cache.head_position = position;
        self.cache.origin = position;
    }

    fn generate_cache(&mut self, up_to: usize) {
        let mut head_position = self.cache.head_position;

        for element in self.elements.iter().take(up_to).skip(self.cache.up_to) {
            match element {
                TextElement::LetterGroup(group) => {
                    let mut object = ObjectUnmanaged::new(group.sprite.clone());
                    object.show();
                    head_position.x += group.left as i32;
                    object.set_position(head_position);
                    head_position.x += group.width as i32;
                    self.cache.objects.push(object);
                }
                TextElement::WhiteSpace(white) => match white {
                    WhiteSpace::Space => head_position.x += 10,
                    WhiteSpace::NewLine => {
                        head_position.x = self.cache.origin.x;
                        head_position.y += 15;
                    }
                },
            }
        }

        self.cache.head_position = head_position;
        self.cache.up_to = up_to.min(self.elements.len());
    }

    fn draw(&mut self, oam: &mut OamIterator, position: Vector2D<i32>, up_to: usize) {
        if position != self.cache.origin {
            self.reset_cache(position);
        }

        self.generate_cache(up_to);

        for (obj, slot) in self.cache.objects.iter().zip(oam) {
            slot.set(obj);
        }
    }
}

struct WorkingLetter {
    dynamic: DynamicSprite,
    // the x offset of the current letter with respect to the start of the current letter group
    x_position: i32,
    // where to render the letter from x_min to x_max
    x_offset: i32,
}

impl WorkingLetter {
    fn new(size: Size) -> Self {
        Self {
            dynamic: DynamicSprite::new(size),
            x_position: 0,
            x_offset: 0,
        }
    }

    fn reset(&mut self) {
        self.x_position = 0;
        self.x_offset = 0;
        self.dynamic.clear(0);
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

pub struct BufferedWordRender<'font> {
    word_render: WordRender<'font>,
    block: TextBlock,
    buffered_chars: VecDeque<char>,
}

impl Write for BufferedWordRender<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for char in s.chars() {
            self.buffered_chars.push_back(char);
        }

        Ok(())
    }
}

impl<'font> BufferedWordRender<'font> {
    #[must_use]
    pub fn new(font: &'font Font, config: Configuration) -> Self {
        BufferedWordRender {
            word_render: WordRender::new(font, config),
            block: TextBlock {
                elements: Vec::new(),
                cache: CachedRender {
                    objects: Vec::new(),
                    up_to: 0,
                    head_position: (0, 0).into(),
                    origin: (0, 0).into(),
                },
            },
            buffered_chars: VecDeque::new(),
        }
    }
}

impl BufferedWordRender<'_> {
    pub fn process(&mut self) {
        if let Some(char) = self.buffered_chars.pop_front() {
            if char == '\n' {
                if let Some(group) = self.word_render.finalise_letter() {
                    self.block.elements.push(TextElement::LetterGroup(group));
                }
                self.block
                    .elements
                    .push(TextElement::WhiteSpace(WhiteSpace::NewLine));
            } else if char == ' ' {
                if let Some(group) = self.word_render.finalise_letter() {
                    self.block.elements.push(TextElement::LetterGroup(group));
                }
                self.block
                    .elements
                    .push(TextElement::WhiteSpace(WhiteSpace::Space));
            } else if let Some(group) = self.word_render.render_char(char) {
                self.block.elements.push(TextElement::LetterGroup(group));
            }
        }
    }

    pub fn draw_partial(
        &mut self,
        oam: &mut OamIterator,
        position: Vector2D<i32>,
        num_groups: usize,
    ) {
        while self.block.elements.len() < num_groups && !self.buffered_chars.is_empty() {
            self.process();
        }

        self.block.draw(oam, position, num_groups);
    }

    pub fn draw(&mut self, oam: &mut OamIterator, position: Vector2D<i32>) {
        while !self.buffered_chars.is_empty() {
            self.process();
        }

        self.block.draw(oam, position, usize::MAX);
    }
}

struct WordRender<'font> {
    font: &'font Font,
    working: Working,
    config: Configuration,
}

struct Working {
    letter: WorkingLetter,
}

impl<'font> WordRender<'font> {
    #[must_use]
    fn new(font: &'font Font, config: Configuration) -> Self {
        WordRender {
            font,
            working: Working {
                letter: WorkingLetter::new(config.sprite_size),
            },
            config,
        }
    }
}

impl WordRender<'_> {
    #[must_use]
    fn finalise_letter(&mut self) -> Option<LetterGroup> {
        if self.working.letter.x_offset == 0 {
            return None;
        }

        let sprite = self
            .working
            .letter
            .dynamic
            .to_vram(self.config.palette.clone());
        let group = LetterGroup {
            sprite,
            width: self.working.letter.x_offset as u16,
            left: self.working.letter.x_position as i16,
        };
        self.working.letter.reset();

        Some(group)
    }

    #[must_use]
    fn render_char(&mut self, c: char) -> Option<LetterGroup> {
        let font_letter = self.font.letter(c);

        // uses more than the sprite can hold
        let group = if self.working.letter.x_offset + font_letter.width as i32
            > self.config.sprite_size.to_width_height().0 as i32
        {
            self.finalise_letter()
        } else {
            None
        };

        if self.working.letter.x_offset == 0 {
            self.working.letter.x_position = font_letter.xmin as i32;
        } else {
            self.working.letter.x_offset += font_letter.xmin as i32;
        }

        let y_position =
            self.font.ascent() - font_letter.height as i32 - font_letter.ymin as i32 + 4;

        for y in 0..font_letter.height as usize {
            for x in 0..font_letter.width as usize {
                let rendered = font_letter.bit_absolute(x, y);
                if rendered {
                    self.working.letter.dynamic.set_pixel(
                        x + self.working.letter.x_offset as usize,
                        (y_position + y as i32) as usize,
                        1,
                    );
                }
            }
        }

        self.working.letter.x_offset += font_letter.advance_width as i32;

        group
    }
}
