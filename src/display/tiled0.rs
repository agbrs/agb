use crate::memory_mapped::MemoryMapped1DArray;

use super::{set_graphics_mode, DisplayMode};

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };
const PALETTE_SPRITE: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0200) };

#[non_exhaustive]
pub struct Tiled0 {}

impl Tiled0 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Tiled0);
        Tiled0 {}
    }

    pub fn set_sprite_palette(&mut self, index: u8, colour: u16) {
        PALETTE_SPRITE.set(index as usize, colour)
    }
    pub fn set_background_palette(&mut self, index: u8, colour: u16) {
        PALETTE_BACKGROUND.set(index as usize, colour)
    }
}
