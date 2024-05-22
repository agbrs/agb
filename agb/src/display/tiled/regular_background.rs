use core::{
    alloc::{Allocator, Layout},
    ptr::NonNull,
};

use alloc::{vec, vec::Vec};

use crate::display::Priority;

use super::{ScreenblockAllocator, Tile, TileFormat, SCREENBLOCK_SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum RegularBackgroundSize {
    Background32x32 = 0,
    Background64x32 = 1,
    Background32x64 = 2,
    Background64x64 = 3,
}

impl RegularBackgroundSize {
    fn size_in_bytes(self) -> usize {
        self.num_tiles() * 2
    }

    fn layout(self) -> Layout {
        Layout::from_size_align(self.size_in_bytes(), SCREENBLOCK_SIZE).unwrap()
    }

    fn num_tiles(self) -> usize {
        match self {
            RegularBackgroundSize::Background32x32 => 32 * 32,
            RegularBackgroundSize::Background64x32 => 64 * 32,
            RegularBackgroundSize::Background32x64 => 32 * 64,
            RegularBackgroundSize::Background64x64 => 64 * 64,
        }
    }
}

pub struct RegularBackgroundTiles {
    priority: Priority,
    size: RegularBackgroundSize,
    colours: TileFormat,

    tiles: Vec<Tile>,
    is_dirty: bool,

    screenblock_ptr: NonNull<[u8]>,
}

impl RegularBackgroundTiles {
    pub fn new(priority: Priority, size: RegularBackgroundSize, colours: TileFormat) -> Self {
        let screenblock_ptr = ScreenblockAllocator
            .allocate(size.layout())
            .expect("Not enough space to allocate for background");

        Self {
            priority,
            size,
            colours,

            tiles: vec![Tile::default(); size.num_tiles()],
            is_dirty: true,

            screenblock_ptr,
        }
    }
}

impl Drop for RegularBackgroundTiles {
    fn drop(&mut self) {
        unsafe { ScreenblockAllocator.deallocate(self.screenblock_ptr.cast(), self.size.layout()) };

        // TODO: Deallocate the tiles
    }
}
