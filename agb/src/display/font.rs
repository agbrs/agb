use core::fmt::{Error, Write};

use crate::fixnum::Vector2D;
use crate::hash_map::HashMap;

use super::tiled::{DynamicTile, RegularMap, TileSetting, VRamManager};

pub struct FontLetter {
    width: u8,
    height: u8,
    data: &'static [u8],
    xmin: i8,
    ymin: i8,
    advance_width: u8,
}

impl FontLetter {
    #[must_use]
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
    #[must_use]
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
            tiles: Default::default(),
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
    tiles: HashMap<(i32, i32), DynamicTile<'a>>,
}

impl<'a> Write for TextRenderer<'a> {
    fn write_str(&mut self, text: &str) -> Result<(), Error> {
        for c in text.chars() {
            if c == '\n' {
                self.current_y_pos += self.font.line_height;
                self.current_x_pos = 0;
                continue;
            }

            let letter = self.font.letter(c);

            self.render_letter(letter);

            self.current_x_pos += i32::from(letter.advance_width);
        }

        Ok(())
    }
}

fn div_ceil(quotient: i32, divisor: i32) -> i32 {
    (quotient + divisor - 1) / divisor
}

impl<'a> TextRenderer<'a> {
    fn render_letter(&mut self, letter: &FontLetter) {
        let vram_manager = &mut self.vram_manager;
        let foreground_colour = self.foreground_colour;
        let background_colour = self.background_colour;

        let x_start = (self.current_x_pos + i32::from(letter.xmin)).max(0);
        let y_start = self.current_y_pos + self.font.ascent
            - i32::from(letter.height)
            - i32::from(letter.ymin);

        let x_tile_start = x_start / 8;
        let y_tile_start = y_start / 8;

        let letter_offset_x = x_start.rem_euclid(8);
        let letter_offset_y = y_start.rem_euclid(8);

        let x_tiles = div_ceil(i32::from(letter.width) + letter_offset_x, 8);
        let y_tiles = div_ceil(i32::from(letter.height) + letter_offset_y, 8);

        for letter_y_tile in 0..(y_tiles + 1) {
            let letter_y_start = 0.max(letter_offset_y - 8 * letter_y_tile) + 8 * letter_y_tile;
            let letter_y_end =
                (letter_offset_y + i32::from(letter.height)).min((letter_y_tile + 1) * 8);

            let tile_y = y_tile_start + letter_y_tile;

            for letter_x_tile in 0..(x_tiles + 1) {
                let letter_x_start = 0.max(letter_offset_x - 8 * letter_x_tile) + 8 * letter_x_tile;
                let letter_x_end =
                    (letter_offset_x + i32::from(letter.width)).min((letter_x_tile + 1) * 8);

                let tile_x = x_tile_start + letter_x_tile;

                let mut masks = [0u32; 8];
                let mut zero = true;

                for letter_y in letter_y_start..letter_y_end {
                    let y = letter_y - letter_offset_y;

                    for letter_x in letter_x_start..letter_x_end {
                        let x = letter_x - letter_offset_x;
                        let pos = x + y * i32::from(letter.width);
                        let px_line = letter.data[(pos / 8) as usize];
                        let px = (px_line >> (pos & 7)) & 1;

                        if px != 0 {
                            masks[(letter_y & 7) as usize] |=
                                u32::from(foreground_colour) << ((letter_x & 7) * 4);
                            zero = false;
                        }
                    }
                }

                if !zero {
                    let tile = self.tiles.entry((tile_x, tile_y)).or_insert_with(|| {
                        vram_manager.new_dynamic_tile().fill_with(background_colour)
                    });

                    for (i, tile_data_line) in tile.tile_data.iter_mut().enumerate() {
                        *tile_data_line |= masks[i];
                    }
                }
            }
        }
    }

    pub fn commit(mut self) {
        let tiles = core::mem::take(&mut self.tiles);

        for ((x, y), tile) in tiles.into_iter() {
            self.bg.set_tile(
                self.vram_manager,
                (self.tile_pos.x + x as u16, self.tile_pos.y + y as u16).into(),
                &tile.tile_set(),
                TileSetting::from_raw(tile.tile_index()),
            );
            self.vram_manager.remove_dynamic_tile(tile);
        }
    }
}

impl<'a> Drop for TextRenderer<'a> {
    fn drop(&mut self) {
        let tiles = core::mem::take(&mut self.tiles);

        for (_, tile) in tiles.into_iter() {
            self.vram_manager.remove_dynamic_tile(tile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::tiled::TiledMap;
    const FONT: Font = crate::include_font!("examples/font/yoster.ttf", 12);

    #[test_case]
    fn font_display(gba: &mut crate::Gba) {
        let (gfx, mut vram) = gba.display.video.tiled0();

        let mut bg = gfx.background(
            crate::display::Priority::P0,
            crate::display::tiled::RegularBackgroundSize::Background32x32,
        );

        vram.set_background_palette_raw(&[
            0x0000, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ]);

        let background_tile = vram.new_dynamic_tile().fill_with(0);

        for y in 0..20u16 {
            for x in 0..30u16 {
                bg.set_tile(
                    &mut vram,
                    (x, y).into(),
                    &background_tile.tile_set(),
                    TileSetting::from_raw(background_tile.tile_index()),
                );
            }
        }

        vram.remove_dynamic_tile(background_tile);

        let mut writer = FONT.render_text((0u16, 3u16).into(), 1, 2, &mut bg, &mut vram);

        writeln!(&mut writer, "Hello, World!").unwrap();
        writeln!(&mut writer, "This is a font rendering example").unwrap();

        writer.commit();

        bg.commit(&mut vram);
        bg.show();

        crate::test_runner::assert_image_output("examples/font/font-test-output.png");
    }
}
