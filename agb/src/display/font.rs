use core::fmt::{Error, Write};

use crate::fixnum::Vector2D;
use crate::hash_map::HashMap;

use super::tiled::{RegularMap, TileSetting, VRamManager};

pub struct FontLetter {
    width: u8,
    height: u8,
    data: &'static [u8],
    xmin: i8,
    ymin: i8,
    advance_width: u8,
}

impl FontLetter {
    pub const fn new(
        width: u8,
        height: u8,
        data: &'static [u8],
        xmin: i8,
        ymin: i8,
        advance_width: u8,
    ) -> Self {
        Self {
            width,
            height,
            data,
            xmin,
            ymin,
            advance_width,
        }
    }
}

pub struct Font {
    letters: &'static [FontLetter],
    line_height: i32,
    ascent: i32,
}

impl Font {
    pub const fn new(letters: &'static [FontLetter], line_height: i32, ascent: i32) -> Self {
        Self {
            letters,
            line_height,
            ascent,
        }
    }

    fn letter(&self, letter: char) -> &'static FontLetter {
        &self.letters[letter as usize]
    }
}

impl Font {
    pub fn render_text<'a>(
        &'a self,
        tile_pos: Vector2D<u16>,
        foreground_colour: u8,
        background_colour: u8,
        bg: &'a mut RegularMap,
        vram_manager: &'a mut VRamManager,
    ) -> TextRenderer<'a> {
        TextRenderer {
            current_x_pos: 0,
            current_y_pos: 0,
            font: self,
            tile_pos,
            vram_manager,
            bg,
            background_colour,
            foreground_colour,
        }
    }
}

pub struct TextRenderer<'a> {
    current_x_pos: i32,
    current_y_pos: i32,
    font: &'a Font,
    tile_pos: Vector2D<u16>,
    vram_manager: &'a mut VRamManager,
    bg: &'a mut RegularMap,
    background_colour: u8,
    foreground_colour: u8,
}

impl<'a> Write for TextRenderer<'a> {
    fn write_str(&mut self, text: &str) -> Result<(), Error> {
        let mut tiles = HashMap::new();

        let vram_manager = &mut self.vram_manager;
        let foreground_colour = self.foreground_colour;
        let background_colour = self.background_colour;

        let mut render_pixel = |x: u16, y: u16| {
            let tile_x = (x / 8) as usize;
            let tile_y = (y / 8) as usize;
            let inner_x = x % 8;
            let inner_y = y % 8;

            let colour = foreground_colour as u32;

            let index = (inner_x + inner_y * 8) as usize;

            let tile = tiles
                .entry((tile_x, tile_y))
                .or_insert_with(|| vram_manager.new_dynamic_tile().fill_with(background_colour));

            tile.tile_data[index / 8] |= colour << ((index % 8) * 4);
        };

        for c in text.chars() {
            if c == '\n' {
                self.current_y_pos += self.font.line_height;
                self.current_x_pos = 0;
                continue;
            }

            let letter = self.font.letter(c);

            let x_start = (self.current_x_pos + letter.xmin as i32).max(0);
            let y_start =
                self.current_y_pos + self.font.ascent - letter.height as i32 - letter.ymin as i32;

            for letter_y in 0..(letter.height as i32) {
                for letter_x in 0..(letter.width as i32) {
                    let x = x_start + letter_x;
                    let y = y_start + letter_y;

                    let px = letter.data[(letter_x + letter_y * letter.width as i32) as usize];

                    if px > 100 {
                        render_pixel(x as u16, y as u16);
                    }
                }
            }

            self.current_x_pos += letter.advance_width as i32;
        }

        for ((x, y), tile) in tiles.into_iter() {
            self.bg.set_tile(
                self.vram_manager,
                (self.tile_pos.x + x as u16, self.tile_pos.y + y as u16).into(),
                &tile.tile_set(),
                TileSetting::from_raw(tile.tile_index()),
            );
            self.vram_manager.remove_dynamic_tile(tile);
        }

        Ok(())
    }
}
