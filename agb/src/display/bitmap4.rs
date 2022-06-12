use crate::memory_mapped::{MemoryMapped1DArray, MemoryMapped2DArray};

use super::{
    set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, DISPLAY_CONTROL,
    HEIGHT, WIDTH,
};

const BITMAP_PAGE_FRONT_MODE_4: MemoryMapped2DArray<
    u16,
    { (WIDTH / 2) as usize },
    { HEIGHT as usize },
> = unsafe { MemoryMapped2DArray::new(0x600_0000) };
const BITMAP_PAGE_BACK_MODE_4: MemoryMapped2DArray<
    u16,
    { (WIDTH / 2) as usize },
    { HEIGHT as usize },
> = unsafe { MemoryMapped2DArray::new(0x600_A000) };
const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };

#[derive(Clone, Copy)]
pub enum Page {
    Front = 0,
    Back = 1,
}

#[non_exhaustive]
pub struct Bitmap4 {}

impl Bitmap4 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Bitmap4);
        set_graphics_settings(GraphicsSettings::LAYER_BG2);
        Bitmap4 {}
    }

    /// Draws point on specified page at (x, y) coordinates with colour index
    /// whose colour is specified in the background palette. Panics if (x, y) is
    /// out of the bounds of the screen.
    pub fn draw_point_page(&mut self, x: i32, y: i32, colour: u8, page: Page) {
        let addr = match page {
            Page::Front => BITMAP_PAGE_FRONT_MODE_4,
            Page::Back => BITMAP_PAGE_BACK_MODE_4,
        };

        let x_in_screen = (x / 2) as usize;
        let y_in_screen = y as usize;

        let c = addr.get(x_in_screen, y_in_screen);
        if x & 0b1 != 0 {
            addr.set(x_in_screen, y_in_screen, c | u16::from(colour) << 8);
        } else {
            addr.set(x_in_screen, y_in_screen, c | u16::from(colour));
        }
    }

    /// Draws point on the non-current page at (x, y) coordinates with colour
    /// index whose colour is specified in the background palette. Panics if (x,
    /// y) is out of the bounds of the screen.
    pub fn draw_point(&mut self, x: i32, y: i32, colour: u8) {
        let disp = DISPLAY_CONTROL.get();

        // get other page
        let page = if disp & GraphicsSettings::PAGE_SELECT.bits() != 0 {
            Page::Front
        } else {
            Page::Back
        };

        self.draw_point_page(x, y, colour, page);
    }

    /// Sets the colour of colour index in the background palette.
    pub fn set_palette_entry(&mut self, entry: u32, colour: u16) {
        PALETTE_BACKGROUND.set(entry as usize, colour);
    }

    /// Flips page, changing the Gameboy advance to draw the contents of the
    /// other page
    pub fn flip_page(&mut self) {
        let disp = DISPLAY_CONTROL.get();
        let swapped = disp ^ GraphicsSettings::PAGE_SELECT.bits();
        DISPLAY_CONTROL.set(swapped);
    }
}
