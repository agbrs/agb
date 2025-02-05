use crate::memory_mapped::MemoryMapped;

use bilge::prelude::*;

use tiled::{BackgroundFrame, TiledBackground};

use self::{
    blend::Blend,
    object::{initilise_oam, Oam, OamFrame},
    window::Windows,
};

/// Graphics mode 3. Bitmap mode that provides a 16-bit colour framebuffer.
pub(crate) mod bitmap3;
/// Test logo of agb.
pub mod example_logo;
pub mod object;
/// Palette type.
pub mod palette16;
/// Data produced by agb-image-converter
pub mod tile_data;
/// Graphics mode 0. Four regular backgrounds.
pub mod tiled;

pub mod affine;
pub mod blend;
pub mod window;

pub mod font;
pub use font::{Font, FontLetter};

const DISPLAY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0000) };
pub(crate) const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
const VCOUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0006) };

/// Width of the Gameboy advance screen in pixels
pub const WIDTH: i32 = 240;
/// Height of the Gameboy advance screen in pixels
pub const HEIGHT: i32 = 160;

#[non_exhaustive]
/// Manages distribution of display modes, obtained from the gba struct
pub struct Display {
    pub window: WindowDist,
    pub blend: BlendDist,
    pub graphics: GraphicsDist,
}

#[non_exhaustive]
pub struct GraphicsDist;

impl GraphicsDist {
    pub fn get(&mut self) -> Graphics<'_> {
        unsafe { initilise_oam() };
        Graphics::new(Oam::new(), unsafe { TiledBackground::new() })
    }
}

pub struct Graphics<'gba> {
    oam: Oam<'gba>,
    tiled: TiledBackground<'gba>,
}

impl<'gba> Graphics<'gba> {
    fn new(oam: Oam<'gba>, tiled: TiledBackground<'gba>) -> Self {
        Self { oam, tiled }
    }

    pub fn frame(&mut self) -> GraphicsFrame<'_> {
        GraphicsFrame {
            oam_frame: self.oam.frame(),
            bg_frame: self.tiled.iter(),
        }
    }
}

pub struct GraphicsFrame<'frame> {
    pub(crate) oam_frame: OamFrame<'frame>,
    pub(crate) bg_frame: BackgroundFrame<'frame>,
}

impl GraphicsFrame<'_> {
    pub fn commit(self) {
        self.oam_frame.commit();
        self.bg_frame.commit();
    }
}

#[non_exhaustive]
pub struct WindowDist;

impl WindowDist {
    pub fn get(&mut self) -> Windows<'_> {
        Windows::new()
    }
}

#[non_exhaustive]
pub struct BlendDist;

impl BlendDist {
    pub fn get(&mut self) -> Blend<'_> {
        Blend::new()
    }
}

impl Display {
    pub(crate) const unsafe fn new() -> Self {
        Display {
            graphics: GraphicsDist,
            window: WindowDist,
            blend: BlendDist,
        }
    }
}

/// Waits until vblank using a busy wait loop, this should almost never be used.
/// I only say almost because whilst I don't believe there to be a reason to use
/// this I can't rule it out.
pub fn busy_wait_for_vblank() {
    while VCOUNT.get() >= 160 {}
    while VCOUNT.get() < 160 {}
}

/// The priority of a background layer or object. A higher priority should be
/// thought of as rendering first, and so is behind that of a lower priority.
/// For an equal priority background layer and object, the background has a
/// higher priority and therefore is behind the object.
#[bitsize(2)]
#[derive(FromBits, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Priority {
    #[default]
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}
