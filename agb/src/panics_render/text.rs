use core::fmt::Write;

use crate::{
    display::{bitmap3::Bitmap3, Font, HEIGHT, WIDTH},
    fixnum::Vector2D,
};

static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

pub struct BitmapTextRender<'bitmap, 'gba> {
    head_position: Vector2D<i32>,
    start_x: i32,
    bitmap: &'bitmap mut Bitmap3<'gba>,
    colour: u16,
    previous_char: Option<char>,
}

impl<'bitmap, 'gba> BitmapTextRender<'bitmap, 'gba> {
    pub fn new(
        bitmap: &'bitmap mut Bitmap3<'gba>,
        position: Vector2D<i32>,
        start_colour: u16,
    ) -> Self {
        Self {
            head_position: position,
            start_x: position.x,
            bitmap,
            colour: start_colour,
            previous_char: None,
        }
    }

    fn render_letter(&mut self, c: char) {
        let letter = FONT.letter(c);

        self.head_position.x += letter.xmin as i32
            + self
                .previous_char
                .take()
                .map_or(0, |c| letter.kerning_amount(c));
        self.previous_char = Some(c);

        if self.head_position.x + letter.width as i32 >= WIDTH {
            self.newline();
        }

        if self.head_position.y + letter.height as i32 >= HEIGHT {
            return;
        }

        let y_position_start =
            self.head_position.y + FONT.ascent() - letter.height as i32 - letter.ymin as i32;

        for y in 0..letter.height as usize {
            for x in 0..letter.width as usize {
                let rendered = letter.bit_absolute(x, y);
                let x = x as i32 + self.head_position.x;
                let y = y as i32 + y_position_start;

                if rendered && (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y) {
                    self.bitmap.draw_point(x, y, self.colour);
                }
            }
        }

        self.head_position.x += letter.advance_width as i32;
    }

    fn render_char(&mut self, c: char) {
        match c {
            '\n' => {
                self.newline();
            }
            ' ' => {
                self.head_position.x += FONT.letter(' ').advance_width as i32;
            }
            letter => self.render_letter(letter),
        }
    }

    fn newline(&mut self) {
        self.head_position.x = self.start_x;
        self.head_position.y += FONT.line_height();
    }
}

impl<'bitmap, 'gba> Write for BitmapTextRender<'bitmap, 'gba> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.render_char(c);
        }

        Ok(())
    }
}
