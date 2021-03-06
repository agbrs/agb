use crate::{
    memory_mapped::{MemoryMapped, MemoryMapped1DArray, MemoryMapped2DArray},
    single::{Single, SingleToken},
};
use bitflags::bitflags;
use core::convert::TryInto;

const DISPLAY_CONTROL: MemoryMapped<u16> = MemoryMapped::new(0x0400_0000);
const DISPLAY_STATUS: MemoryMapped<u16> = MemoryMapped::new(0x0400_0004);
const VCOUNT: MemoryMapped<u16> = MemoryMapped::new(0x0400_0006);

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> = MemoryMapped1DArray::new(0x0500_0000);
const PALETTE_SPRITE: MemoryMapped1DArray<u16, 256> = MemoryMapped1DArray::new(0x0500_0200);

const BITMAP_MODE_3: MemoryMapped2DArray<u16, { WIDTH as usize }, { HEIGHT as usize }> =
    MemoryMapped2DArray::new(0x600_0000);

const BITMAP_PAGE_FRONT_MODE_4: MemoryMapped2DArray<
    u16,
    { (WIDTH / 2) as usize },
    { HEIGHT as usize },
> = MemoryMapped2DArray::new(0x600_0000);
const BITMAP_PAGE_BACK_MODE_4: MemoryMapped2DArray<
    u16,
    { (WIDTH / 2) as usize },
    { HEIGHT as usize },
> = MemoryMapped2DArray::new(0x600_A000);

pub const WIDTH: i32 = 240;
pub const HEIGHT: i32 = 160;

pub enum DisplayMode {
    Tiled0 = 0,
    Tiled1 = 1,
    Tiled2 = 2,
    Bitmap3 = 3,
    Bitmap4 = 4,
    Bitmap5 = 5,
}

pub enum Page {
    Front = 0,
    Back = 1,
}

bitflags! {
    pub struct GraphicsSettings: u16 {
        const PAGE_SELECT = 1 << 0x4;
        const OAM_HBLANK = 1 << 0x5;
        const SPRITE1_D = 1 << 0x6;
        const SCREEN_BLANK = 1 << 0x7;
        const LAYER_BG0 = 1 << 0x8;
        const LAYER_BG1 = 1 << 0x9;
        const LAYER_BG2 = 1 << 0xA;
        const LAYER_BG3 = 1  << 0xB;
        const LAYER_OBJ = 1 << 0xC;
        const WINDOW0 = 1 << 0xD;
        const WINDOW1 = 1 << 0xE;
        const WINDOW_OBJECT = 1 << 0xF;
    }
}

pub struct Display {
    in_mode: Single,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    pub(crate) const fn new() -> Self {
        Display {
            in_mode: Single::new(),
        }
    }

    pub fn bitmap3(&self) -> Bitmap3 {
        Bitmap3::new(
            self.in_mode
                .take()
                .expect("Cannot create new mode as mode already taken"),
        )
    }
    pub fn bitmap4(&self) -> Bitmap4 {
        Bitmap4::new(
            self.in_mode
                .take()
                .expect("Cannot create new mode as mode already taken"),
        )
    }
}

pub struct Bitmap3<'a> {
    _in_mode: SingleToken<'a>,
}

impl<'a> Bitmap3<'a> {
    fn new(in_mode: SingleToken<'a>) -> Self {
        set_graphics_mode(DisplayMode::Bitmap3);
        set_graphics_settings(GraphicsSettings::LAYER_BG2);
        Bitmap3 { _in_mode: in_mode }
    }
    pub fn draw_point(&self, x: i32, y: i32, colour: u16) {
        let x = x.try_into().unwrap();
        let y = y.try_into().unwrap();
        BITMAP_MODE_3.set(x, y, colour)
    }
}

pub struct Bitmap4<'a> {
    _in_mode: SingleToken<'a>,
}

impl<'a> Bitmap4<'a> {
    fn new(in_mode: SingleToken<'a>) -> Self {
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

fn set_graphics_mode(mode: DisplayMode) {
    let current = DISPLAY_CONTROL.get();
    let current = current & (!0b111);
    let s = current | (mode as u16 & 0b111);

    DISPLAY_CONTROL.set(s);
}

pub fn set_graphics_settings(settings: GraphicsSettings) {
    let current = DISPLAY_CONTROL.get();
    // preserve display mode
    let current = current & 0b111;
    let s = settings.bits() | current;

    DISPLAY_CONTROL.set(s);
}

#[allow(non_snake_case)]
pub fn busy_wait_for_VBlank() {
    while VCOUNT.get() >= 160 {}
    while VCOUNT.get() < 160 {}
}

#[allow(non_snake_case)]
pub fn enable_VBlank_interrupt() {
    let status = DISPLAY_STATUS.get() | (1 << 3);
    DISPLAY_STATUS.set(status);
}

#[allow(non_snake_case)]
pub fn wait_for_VBlank() {
    crate::syscall::wait_for_VBlank();
}
