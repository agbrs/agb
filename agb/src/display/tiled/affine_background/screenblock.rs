use core::ptr::NonNull;

use alloc::alloc::Allocator;

use crate::display::tiled::{SCREENBLOCK_SIZE, ScreenblockAllocator, VRAM_START};

use super::{AffineBackgroundSize, Tiles};

pub(crate) struct AffineBackgroundScreenBlock {
    ptr: NonNull<u8>,
    size: AffineBackgroundSize,
}

impl AffineBackgroundScreenBlock {
    pub(crate) fn new(size: AffineBackgroundSize) -> Self {
        let screenblock_ptr = ScreenblockAllocator
            .allocate(size.layout())
            .expect("Not enough space to allocate for affine background")
            .cast();

        Self {
            ptr: screenblock_ptr,
            size,
        }
    }

    pub(crate) unsafe fn copy_tiles(&self, tiles: &Tiles) {
        unsafe {
            self.ptr
                .as_ptr()
                .cast::<u8>()
                .copy_from_nonoverlapping(tiles.as_ptr(), self.size.num_tiles());
        }
    }

    pub(crate) fn size(&self) -> AffineBackgroundSize {
        self.size
    }

    pub(crate) fn screen_base_block(&self) -> u16 {
        let screenblock_location = self.ptr.as_ptr() as usize;
        ((screenblock_location - VRAM_START) / SCREENBLOCK_SIZE) as u16
    }
}
