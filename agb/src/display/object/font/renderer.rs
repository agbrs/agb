use crate::display::{
    object::{DynamicSprite, PaletteVram, Size, SpriteVram},
    Font,
};

use super::ChangeColour;

struct WorkingLetter {
    dynamic: DynamicSprite,
    // where to render the letter from x_min to x_max
    x_offset: i32,
}

impl WorkingLetter {
    fn new(size: Size) -> Self {
        Self {
            dynamic: DynamicSprite::new(size),
            x_offset: 0,
        }
    }

    fn reset(&mut self) {
        self.x_offset = 0;
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

pub(crate) struct WordRender {
    working: WorkingLetter,
    config: Configuration,
    colour: usize,
    start_index_of_letter: usize,

    previous_character: Option<char>,
    explicit_break_on: Option<fn(char) -> bool>,
}

impl WordRender {
    #[must_use]
    pub(crate) fn new(config: Configuration, explicit_break_on: Option<fn(char) -> bool>) -> Self {
        WordRender {
            working: WorkingLetter::new(config.sprite_size),
            config,
            colour: 1,
            previous_character: None,
            start_index_of_letter: 0,
            explicit_break_on,
        }
    }

    #[must_use]
    pub(crate) fn finalise_letter(
        &mut self,
        index_of_character: usize,
    ) -> Option<(usize, SpriteVram)> {
        if self.working.x_offset == 0 {
            return None;
        }

        let mut new_sprite = DynamicSprite::new(self.config.sprite_size);
        core::mem::swap(&mut self.working.dynamic, &mut new_sprite);
        let sprite = new_sprite.to_vram(self.config.palette.clone());
        let start_index = self.start_index_of_letter;
        self.working.reset();
        self.start_index_of_letter = index_of_character;

        Some((start_index, sprite))
    }

    #[must_use]
    pub(crate) fn render_char(
        &mut self,
        font: &Font,
        c: char,
        index_of_character: usize,
    ) -> Option<(usize, SpriteVram)> {
        if let Some(next_colour) = ChangeColour::try_from_char(c) {
            self.colour = next_colour.0 as usize;
            return None;
        }

        let font_letter: &crate::display::FontLetter = font.letter(c);

        if let Some(previous_character) = self.previous_character {
            self.working.x_offset += font_letter.kerning_amount(previous_character);
        }
        self.previous_character = Some(c);

        // uses more than the sprite can hold
        let group = if self.working.x_offset + font_letter.width as i32
            > self.config.sprite_size.to_width_height().0 as i32
            || self.explicit_break_on.map(|x| x(c)).unwrap_or_default()
        {
            self.finalise_letter(index_of_character)
        } else {
            None
        };

        if self.working.x_offset != 0 {
            self.working.x_offset += font_letter.xmin as i32;
        }

        let y_position = font.ascent() - font_letter.height as i32 - font_letter.ymin as i32;

        for y in 0..font_letter.height as usize {
            for x in 0..font_letter.width as usize {
                let rendered = font_letter.bit_absolute(x, y);
                if rendered {
                    self.working.dynamic.set_pixel(
                        x + self.working.x_offset as usize,
                        (y_position + y as i32) as usize,
                        self.colour,
                    );
                }
            }
        }

        self.working.x_offset += font_letter.advance_width as i32;

        group
    }
}
