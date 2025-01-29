use core::fmt::{Error, Write};

use crate::fixnum::Vector2D;
use crate::hash_map::HashMap;

use super::tiled::{DynamicTile, RegularBackgroundTiles};

/// The text renderer renders a variable width fixed size
/// bitmap font using dynamic tiles as a rendering surface.
/// Does not support any unicode features.
/// For usage see the `text_render.rs` example
pub struct FontLetter {
    pub(crate) character: char,
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) data: &'static [u8],
    pub(crate) xmin: i8,
    pub(crate) ymin: i8,
    pub(crate) advance_width: u8,
    kerning_amounts: &'static [(char, i8)],
}

impl FontLetter {
    #[must_use]
    #[allow(clippy::too_many_arguments)] // only used in macro
    pub const fn new(
        character: char,
        width: u8,
        height: u8,
        data: &'static [u8],
        xmin: i8,
        ymin: i8,
        advance_width: u8,
        kerning_amounts: &'static [(char, i8)],
    ) -> Self {
        Self {
            character,
            width,
            height,
            data,
            xmin,
            ymin,
            advance_width,
            kerning_amounts,
        }
    }

    pub(crate) const fn bit_absolute(&self, x: usize, y: usize) -> bool {
        let position = x + y * self.width as usize;
        let byte = self.data[position / 8];
        let bit = position % 8;
        ((byte >> bit) & 1) != 0
    }

    pub(crate) fn kerning_amount(&self, previous_char: char) -> i32 {
        if let Ok(index) = self
            .kerning_amounts
            .binary_search_by_key(&previous_char, |kerning_data| kerning_data.0)
        {
            self.kerning_amounts[index].1 as i32
        } else {
            0
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

    pub(crate) fn letter(&self, letter: char) -> &'static FontLetter {
        let letter = self
            .letters
            .binary_search_by_key(&letter, |letter| letter.character);

        match letter {
            Ok(index) => &self.letters[index],
            Err(_) => &self.letters[0],
        }
    }

    pub(crate) fn ascent(&self) -> i32 {
        self.ascent
    }

    pub(crate) fn line_height(&self) -> i32 {
        self.line_height
    }
}

impl Font {
    #[must_use]
    /// Create renderer starting at the given tile co-ordinates.
    pub fn render_text(&self, tile_pos: impl Into<Vector2D<u16>>) -> TextRenderer<'_> {
        TextRenderer {
            current_x_pos: 0,
            current_y_pos: 0,
            previous_character: None,
            font: self,
            tile_pos: tile_pos.into(),
            tiles: Default::default(),
        }
    }
}

/// Keeps track of the cursor and manages rendered tiles.
pub struct TextRenderer<'a> {
    current_x_pos: i32,
    current_y_pos: i32,
    previous_character: Option<char>,
    font: &'a Font,
    tile_pos: Vector2D<u16>,
    tiles: HashMap<(i32, i32), DynamicTile<'a>>,
}

/// Generated from the renderer for use
/// with `Write` trait methods.
pub struct TextWriter<'a, 'b> {
    foreground_colour: u8,
    background_colour: u8,
    text_renderer: &'a mut TextRenderer<'b>,
    bg: &'a mut RegularBackgroundTiles,
}

impl Write for TextWriter<'_, '_> {
    fn write_str(&mut self, text: &str) -> Result<(), Error> {
        for c in text.chars() {
            self.text_renderer
                .write_char(c, self.foreground_colour, self.background_colour);
        }

        Ok(())
    }
}

impl TextWriter<'_, '_> {
    pub fn commit(self) {
        self.text_renderer.commit(self.bg);
    }
}

fn div_ceil(quotient: i32, divisor: i32) -> i32 {
    (quotient + divisor - 1) / divisor
}

impl<'a, 'b> TextRenderer<'b> {
    pub fn writer(
        &'a mut self,
        foreground_colour: u8,
        background_colour: u8,
        bg: &'a mut RegularBackgroundTiles,
    ) -> TextWriter<'a, 'b> {
        TextWriter {
            text_renderer: self,
            foreground_colour,
            background_colour,
            bg,
        }
    }

    /// Renders a single character creating as many dynamic tiles as needed.
    /// The foreground and background colour are palette indicies.
    fn render_letter(&mut self, letter: &FontLetter, foreground_colour: u8, background_colour: u8) {
        assert!(foreground_colour < 16);
        assert!(background_colour < 16);

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
                    let tile = self
                        .tiles
                        .entry((tile_x, tile_y))
                        .or_insert_with(|| DynamicTile::new().fill_with(background_colour));

                    for (i, tile_data_line) in tile.tile_data.iter_mut().enumerate() {
                        *tile_data_line |= masks[i];
                    }
                }
            }
        }
    }

    /// Commit the dynamic tiles that contain the text to the background.
    pub fn commit(&self, bg: &'a mut RegularBackgroundTiles) {
        for ((x, y), tile) in self.tiles.iter() {
            bg.set_tile(
                (self.tile_pos.x + *x as u16, self.tile_pos.y + *y as u16),
                &tile.tile_set(),
                tile.tile_setting(),
            );
        }
    }

    /// Write another char into the text, moving the cursor as appropriate.
    pub fn write_char(&mut self, c: char, foreground_colour: u8, background_colour: u8) {
        if c == '\n' {
            self.current_y_pos += self.font.line_height;
            self.current_x_pos = 0;
        } else {
            let letter = self.font.letter(c);

            if let Some(previous_character) = self.previous_character {
                self.current_x_pos += letter.kerning_amount(previous_character);
            }
            self.previous_character = Some(c);

            self.render_letter(letter, foreground_colour, background_colour);
            self.current_x_pos += i32::from(letter.advance_width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::tiled::{RegularBackgroundTiles, TileFormat, VRAM_MANAGER};
    static FONT: Font = crate::include_font!("examples/font/yoster.ttf", 12);

    #[test_case]
    fn font_display(gba: &mut crate::Gba) {
        let mut gfx = gba.display.video.tiled();

        let mut bg = RegularBackgroundTiles::new(
            crate::display::Priority::P0,
            crate::display::tiled::RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );

        VRAM_MANAGER.set_background_palette_raw(&[
            0x0000, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ]);

        let background_tile = VRAM_MANAGER.new_dynamic_tile().fill_with(0);

        for y in 0..20u16 {
            for x in 0..30u16 {
                bg.set_tile(
                    (x, y),
                    &background_tile.tile_set(),
                    background_tile.tile_setting(),
                );
            }
        }

        // Test twice to ensure that clearing works
        for _ in 0..2 {
            let mut renderer = FONT.render_text((0u16, 3u16));

            let mut writer = renderer.writer(1, 2, &mut bg);
            write!(&mut writer, "Hello, ").unwrap();

            // Test changing color
            let mut writer = renderer.writer(4, 2, &mut bg);
            writeln!(&mut writer, "World!").unwrap();
            writer.commit();

            // Test writing with same renderer after showing background
            let mut writer = renderer.writer(1, 2, &mut bg);
            writeln!(&mut writer, "This is a font rendering example").unwrap();
            writer.commit();
            bg.commit();

            let mut bg_iter = gfx.iter();
            bg.show(&mut bg_iter);
            bg_iter.commit();

            crate::test_runner::assert_image_output("examples/font/font-test-output.png");
        }
    }
}
