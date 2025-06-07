use core::ptr::NonNull;

use alloc::alloc::Allocator;

use crate::display::tiled::{SCREENBLOCK_SIZE, ScreenblockAllocator, Tile, VRAM_START};

use super::{RegularBackgroundSize, Tiles};

pub(crate) struct RegularBackgroundScreenblock {
    ptr: NonNull<u8>,
    size: RegularBackgroundSize,
}

impl RegularBackgroundScreenblock {
    pub(crate) fn new(size: RegularBackgroundSize) -> Self {
        let screenblock_ptr = ScreenblockAllocator
            .allocate(size.layout())
            .expect("Not enough space to allocate for background")
            .cast();

        Self {
            ptr: screenblock_ptr,
            size,
        }
    }

    pub(crate) fn ptr(&self) -> NonNull<u8> {
        self.ptr
    }

    pub(crate) unsafe fn copy_tiles(&self, tiles: &Tiles) {
        unsafe {
            self.ptr
                .as_ptr()
                .cast::<Tile>()
                .copy_from_nonoverlapping(tiles.as_ptr(), self.size.num_tiles());
        }
    }

    pub(crate) fn size(&self) -> RegularBackgroundSize {
        self.size
    }

    pub(crate) fn screen_base_block(&self) -> u16 {
        let screenblock_location = self.ptr.as_ptr() as usize;
        ((screenblock_location - VRAM_START) / SCREENBLOCK_SIZE) as u16
    }
}

impl Drop for RegularBackgroundScreenblock {
    fn drop(&mut self) {
        unsafe { ScreenblockAllocator.deallocate(self.ptr.cast(), self.size.layout()) };
    }
}
