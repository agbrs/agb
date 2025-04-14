use crate::{interrupt::VBlank, memory_mapped::MemoryMapped};

use alloc::boxed::Box;
use bilge::prelude::*;

use tiled::{BackgroundFrame, DisplayControlRegister, TiledBackground};

use object::{Oam, OamFrame, initilise_oam};

pub use colours::{Rgb, Rgb15, include_colours};
pub use palette16::Palette16;

/// Graphics mode 3. Bitmap mode that provides a 16-bit colour framebuffer.
pub(crate) mod bitmap3;
mod colours;
/// Test logo of agb.
pub mod example_logo;
pub mod object;
/// Palette type.
mod palette16;
/// Data produced by agb-image-converter
pub mod tile_data;
pub mod tiled;

pub mod affine;
mod blend;
mod window;

pub mod font;

const DISPLAY_CONTROL: MemoryMapped<DisplayControlRegister> =
    unsafe { MemoryMapped::new(0x0400_0000) };
pub(crate) const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
const VCOUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0006) };

pub use blend::{
    Blend, BlendAlphaEffect, BlendFadeEffect, BlendObjectTransparency, Layer as BlendLayer,
};

pub use window::{MovableWindow, WinIn, Window, Windows};

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
    others: Others,
}

pub(crate) trait DmaFrame {
    fn commit(&mut self);
    fn cleanup(&mut self);
}

struct Others {
    vblank: VBlank,
    dma: Option<Box<dyn DmaFrame>>,
}

impl<'gba> Graphics<'gba> {
    fn new(oam: Oam<'gba>, tiled: TiledBackground<'gba>, vblank: VBlank) -> Self {
        Self {
            oam,
            tiled,
            others: Others { vblank, dma: None },
        }
    }

    pub fn frame(&mut self) -> GraphicsFrame<'_> {
        GraphicsFrame {
            oam_frame: self.oam.frame(),
            bg_frame: self.tiled.iter(),
            blend: Blend::new(),
            windows: Windows::new(),
            next_dma: None,
            others: &mut self.others,
        }
    }
}

pub struct GraphicsFrame<'frame> {
    pub(crate) oam_frame: OamFrame<'frame>,
    pub(crate) bg_frame: BackgroundFrame<'frame>,
    blend: Blend,
    windows: Windows,
    next_dma: Option<Box<dyn DmaFrame>>,

    others: &'frame mut Others,
}

impl GraphicsFrame<'_> {
    pub fn commit(mut self) {
        self.others.vblank.wait_for_vblank();
        core::mem::swap(&mut self.others.dma, &mut self.next_dma);

        if let Some(mut old) = self.next_dma.take() {
            old.cleanup();
        }

        self.oam_frame.commit();
        self.bg_frame.commit();
        self.blend.commit();
        self.windows.commit();

        if let Some(dma) = self.others.dma.as_mut() {
            dma.commit();
        }
    }

    pub fn blend(&mut self) -> &mut Blend {
        &mut self.blend
    }

    pub fn windows(&mut self) -> &mut Windows {
        &mut self.windows
    }

    pub(crate) fn add_dma<C: DmaFrame + 'static>(&mut self, c: C) {
        self.next_dma = Some(Box::new(c));
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
