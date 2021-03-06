use crate::{
    memory_mapped::{MemoryMapped1DArray, MemoryMapped2DArray},
    single::SingleToken,
};

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

pub enum Page {
    Front = 0,
    Back = 1,
}

pub struct Bitmap4<'a> {
    _in_mode: SingleToken<'a>,
}

impl<'a> Bitmap4<'a> {
    pub(crate) fn new(in_mode: SingleToken<'a>) -> Self {
        set_graphics_mode(DisplayMode::Bitmap4);
        set_graphics_settings(GraphicsSettings::LAYER_BG2);
        Bitmap4 { _in_mode: in_mode }
    }

    pub fn draw_point_page(&self, x: i32, y: i32, colour: u8, page: Page) {
        let addr = match page {
            Page::Front => BITMAP_PAGE_FRONT_MODE_4,
            Page::Back => BITMAP_PAGE_BACK_MODE_4,
        };

        let x_in_screen = (x / 2) as usize;
        let y_in_screen = y as usize;

        let c = addr.get(x_in_screen, y_in_screen);
        if x & 0b1 != 0 {
            addr.set(x_in_screen, y_in_screen, c | (colour as u16) << 8);
        } else {
            addr.set(x_in_screen, y_in_screen, c | colour as u16);
        }
    }

    pub fn draw_point(&self, x: i32, y: i32, colour: u8) {
        let disp = DISPLAY_CONTROL.get();

        let page = if disp & GraphicsSettings::PAGE_SELECT.bits() != 0 {
            Page::Back
        } else {
            Page::Front
        };

        self.draw_point_page(x, y, colour, page)
    }

    pub fn set_palette_entry(&self, entry: u32, colour: u16) {
        PALETTE_BACKGROUND.set(entry as usize, colour);
    }

    pub fn flip_page(&self) {
        let disp = DISPLAY_CONTROL.get();
        let swapped = disp ^ GraphicsSettings::PAGE_SELECT.bits();
        DISPLAY_CONTROL.set(swapped);
    }
}
