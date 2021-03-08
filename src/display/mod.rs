use crate::{
    memory_mapped::MemoryMapped,
    single::{Single, SingleToken},
};
use bitflags::bitflags;

use bitmap3::Bitmap3;
use bitmap4::Bitmap4;

pub mod bitmap3;
pub mod bitmap4;
pub mod tiled0;

const DISPLAY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0000) };
const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
const VCOUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0006) };

bitflags! {
    struct GraphicsSettings: u16 {
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

/// Width of the Gameboy advance screen in pixels
pub const WIDTH: i32 = 240;
/// Height of the Gameboy advance screen in pixels
pub const HEIGHT: i32 = 160;

enum DisplayMode {
    Tiled0 = 0,
    Tiled1 = 1,
    Tiled2 = 2,
    Bitmap3 = 3,
    Bitmap4 = 4,
    Bitmap5 = 5,
}

#[non_exhaustive]
/// Manages distribution of display modes, obtained from the gba struct
pub struct Display {
    pub video: Video,
    pub vblank: VBlankGiver,
}
#[non_exhaustive]
pub struct Video {}

#[non_exhaustive]
pub struct VBlankGiver {}

impl Video {
    /// Bitmap mode that provides a 16-bit colour framebuffer
    pub fn bitmap3(&mut self) -> Bitmap3 {
        unsafe { Bitmap3::new() }
    }

    /// Bitmap 4 provides two 8-bit paletted framebuffers with page switching
    pub fn bitmap4(&mut self) -> Bitmap4 {
        unsafe { bitmap4::Bitmap4::new() }
    }
}

impl VBlankGiver {
    /// Gets a vblank handle where only one can be obtained at a time
    pub fn get(&mut self) -> VBlank {
        unsafe { VBlank::new() }
    }
}

impl Display {
    pub(crate) const unsafe fn new() -> Self {
        Display {
            video: Video {},
            vblank: VBlankGiver {},
        }
    }
}

unsafe fn set_graphics_mode(mode: DisplayMode) {
    let current = DISPLAY_CONTROL.get();
    let current = current & (!0b111);
    let s = current | (mode as u16 & 0b111);

    DISPLAY_CONTROL.set(s);
}

unsafe fn set_graphics_settings(settings: GraphicsSettings) {
    let current = DISPLAY_CONTROL.get();
    // preserve display mode
    let current = current & 0b111;
    let s = settings.bits() | current;

    DISPLAY_CONTROL.set(s);
}

#[allow(non_snake_case)]
/// Waits until vblank using a busy wait loop, this should almost never be used.
/// I only say almost because whilst I don't believe there to be a reason to use
/// this I can't rule it out.
pub fn busy_wait_for_VBlank() {
    while VCOUNT.get() >= 160 {}
    while VCOUNT.get() < 160 {}
}

/// Once obtained, this guarentees that interrupts are enabled and set up to
/// allow for waiting for vblank
pub struct VBlank {}

impl VBlank {
    unsafe fn new() -> Self {
        crate::interrupt::enable_interrupts();
        crate::interrupt::enable(crate::interrupt::Interrupt::VBlank);
        enable_VBlank_interrupt();
        VBlank {}
    }

    #[allow(non_snake_case)]
    /// Waits for VBlank using interrupts. This is the preferred method for
    /// waiting until the next frame.
    pub fn wait_for_VBlank(&self) {
        crate::syscall::wait_for_VBlank();
    }
}

impl Drop for VBlank {
    fn drop(&mut self) {
        unsafe {
            disable_VBlank_interrupt();
            crate::interrupt::disable(crate::interrupt::Interrupt::VBlank);
        }
    }
}

#[allow(non_snake_case)]
unsafe fn enable_VBlank_interrupt() {
    let status = DISPLAY_STATUS.get() | (1 << 3);
    DISPLAY_STATUS.set(status);
}

#[allow(non_snake_case)]
unsafe fn disable_VBlank_interrupt() {
    let status = DISPLAY_STATUS.get() & !(1 << 3);
    DISPLAY_STATUS.set(status);
}
