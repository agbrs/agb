use crate::{interrupt::VBlank, memory_mapped::MemoryMapped};

use bilge::prelude::*;

use tiled::{BackgroundFrame, DisplayControlRegister, TiledBackground};

use self::{
    object::{Oam, OamFrame, initilise_oam},
    window::Windows,
};

pub use palette16::{Palette16, include_palette};

/// Graphics mode 3. Bitmap mode that provides a 16-bit colour framebuffer.
pub(crate) mod bitmap3;
/// Test logo of agb.
pub mod example_logo;
pub mod object;
/// Palette type.
mod palette16;
/// Data produced by agb-image-converter
pub mod tile_data;
/// Graphics mode 0. Four regular backgrounds.
pub mod tiled;

pub mod affine;
mod blend;
pub mod window;

pub mod font;

const DISPLAY_CONTROL: MemoryMapped<DisplayControlRegister> =
    unsafe { MemoryMapped::new(0x0400_0000) };
pub(crate) const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
const VCOUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0006) };

pub use blend::{
    Blend, BlendAlphaEffect, BlendFadeEffect, BlendObjectTransparency, Layer as BlendLayer,
};

/// Width of the Gameboy advance screen in pixels
pub const WIDTH: i32 = 240;
/// Height of the Gameboy advance screen in pixels
pub const HEIGHT: i32 = 160;

#[non_exhaustive]
pub struct GraphicsDist;

impl GraphicsDist {
    pub fn get(&mut self) -> Graphics<'_> {
        unsafe { initilise_oam() };
        Graphics::new(Oam::new(), unsafe { TiledBackground::new() }, VBlank::get())
    }
}

pub struct Graphics<'gba> {
    oam: Oam<'gba>,
    tiled: TiledBackground<'gba>,
    vblank: VBlank,
}

impl<'gba> Graphics<'gba> {
    fn new(oam: Oam<'gba>, tiled: TiledBackground<'gba>, vblank: VBlank) -> Self {
        Self { oam, tiled, vblank }
    }

    pub fn frame(&mut self) -> GraphicsFrame<'_> {
        GraphicsFrame {
            oam_frame: self.oam.frame(),
            bg_frame: self.tiled.iter(),
            blend: Blend::new(),
            windows: Windows::new(),
            vblank: &self.vblank,
        }
    }
}

pub struct GraphicsFrame<'frame> {
    pub(crate) oam_frame: OamFrame<'frame>,
    pub(crate) bg_frame: BackgroundFrame<'frame>,
    blend: Blend,
    windows: Windows,
    vblank: &'frame VBlank,
}

impl GraphicsFrame<'_> {
    pub fn commit(self) {
        self.vblank.wait_for_vblank();

        self.oam_frame.commit();
        self.bg_frame.commit();
        self.blend.commit();
        self.windows.commit();
    }

    pub fn blend(&mut self) -> &mut Blend {
        &mut self.blend
    }

    pub fn windows(&mut self) -> &mut Windows {
        &mut self.windows
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
