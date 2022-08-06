use crate::memory_mapped::MemoryMapped;
use bitflags::bitflags;

use modular_bitfield::BitfieldSpecifier;
use video::Video;

use self::{object::ObjectController, window::Windows};

/// Graphics mode 3. Bitmap mode that provides a 16-bit colour framebuffer.
pub mod bitmap3;
/// Graphics mode 4. Bitmap 4 provides two 8-bit paletted framebuffers with page switching.
pub mod bitmap4;
/// Test logo of agb.
pub mod example_logo;
/// Implements sprites.
pub mod object;
/// Palette type.
pub mod palette16;
/// Data produced by agb-image-converter
pub mod tile_data;
/// Graphics mode 0. Four regular backgrounds.
pub mod tiled;
/// Giving out graphics mode.
pub mod video;

pub mod blend;
pub mod window;

mod font;
pub use font::{Font, FontLetter};

const DISPLAY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0000) };
pub(crate) const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
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

#[allow(dead_code)]
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
    pub object: ObjectDistribution,
    pub window: WindowDist,
}

#[non_exhaustive]
pub struct ObjectDistribution {}

impl ObjectDistribution {
    pub fn get(&mut self) -> ObjectController {
        ObjectController::new()
    }
}

#[non_exhaustive]
pub struct WindowDist {}

impl WindowDist {
    pub fn get(&mut self) -> Windows {
        Windows::new()
    }
}

impl Display {
    pub(crate) const unsafe fn new() -> Self {
        Display {
            video: Video {},
            object: ObjectDistribution {},
            window: WindowDist {},
        }
    }
}

unsafe fn set_graphics_mode(mode: DisplayMode) {
    let current = DISPLAY_CONTROL.get();
    let current = current & (!0b111);
    let s = current | (mode as u16 & 0b111);

    // disable blank screen
    let s = s & !(1 << 7);

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
pub fn busy_wait_for_vblank() {
    while VCOUNT.get() >= 160 {}
    while VCOUNT.get() < 160 {}
}

#[derive(BitfieldSpecifier, Clone, Copy)]
pub enum Priority {
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}
