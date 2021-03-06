use crate::{memory_mapped::MemoryMapped, single::Single};
use bitflags::bitflags;

use bitmap3::Bitmap3;
use bitmap4::Bitmap4;

mod bitmap3;
mod bitmap4;

const DISPLAY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0000) };
const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
const VCOUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0006) };

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

pub struct Display {
    in_mode: Single,
}

impl Display {
    pub(crate) const unsafe fn new() -> Self {
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
        bitmap4::Bitmap4::new(
            self.in_mode
                .take()
                .expect("Cannot create new mode as mode already taken"),
        )
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
