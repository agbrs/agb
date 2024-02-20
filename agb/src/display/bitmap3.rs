use crate::memory_mapped::MemoryMapped2DArray;

use super::{
    set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, HEIGHT, WIDTH,
};

use core::marker::PhantomData;

const BITMAP_MODE_3: MemoryMapped2DArray<u16, { WIDTH as usize }, { HEIGHT as usize }> =
    unsafe { MemoryMapped2DArray::new(0x600_0000) };

#[non_exhaustive]
pub struct Bitmap3<'gba> {
    phantom: PhantomData<&'gba ()>,
}

impl Bitmap3<'_> {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Bitmap3);
        set_graphics_settings(GraphicsSettings::LAYER_BG2);
        Bitmap3 {
            phantom: PhantomData,
        }
    }

    /// Draws point to screen at (x, y) coordinates with colour and panics if
    /// (x, y) is out of the bounds of the screen.
    pub fn draw_point(&mut self, x: i32, y: i32, colour: u16) {
        let x = x.try_into().unwrap();
        let y = y.try_into().unwrap();
        BITMAP_MODE_3.set(x, y, colour);
    }

    #[must_use]
    pub fn read_point(&self, x: i32, y: i32) -> u16 {
        let x = x.try_into().unwrap();
        let y = y.try_into().unwrap();
        BITMAP_MODE_3.get(x, y)
    }

    pub fn clear(&mut self, colour: u16) {
        for y in 0..(HEIGHT as usize) {
            for x in 0..(WIDTH as usize) {
                BITMAP_MODE_3.set(x, y, colour);
            }
        }
    }
}
