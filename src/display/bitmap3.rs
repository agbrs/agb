use crate::memory_mapped::MemoryMapped2DArray;

use super::{
    set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, HEIGHT, WIDTH,
};

use core::convert::TryInto;

const BITMAP_MODE_3: MemoryMapped2DArray<u16, { WIDTH as usize }, { HEIGHT as usize }> =
    unsafe { MemoryMapped2DArray::new(0x600_0000) };

pub struct Bitmap3 {}

impl Bitmap3 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Bitmap3);
        set_graphics_settings(GraphicsSettings::LAYER_BG2);
        Bitmap3 {}
    }

    /// Draws point to screen at (x, y) coordinates with colour and panics if
    /// (x, y) is out of the bounds of the screen.
    pub fn draw_point(&mut self, x: i32, y: i32, colour: u16) {
        let x = x.try_into().unwrap();
        let y = y.try_into().unwrap();
        BITMAP_MODE_3.set(x, y, colour)
    }
}
