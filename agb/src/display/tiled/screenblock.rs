use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::Allocator;

use crate::display::tiled::{
    AffineBackgroundSize, RegularBackgroundSize, SCREENBLOCK_SIZE, ScreenblockAllocator, Tile,
    VRAM_START,
    tiles::{TileInfo, Tiles},
};

pub(crate) struct Screenblock<Size>
where
    Size: ScreenblockSize,
{
    ptr: NonNull<u8>,
    size: Size,
}

pub(crate) trait ScreenblockSize: Copy {
    type TileType: TileInfo;

    fn layout(self) -> Layout;
    fn num_tiles(self) -> usize;
}

impl<Size> Screenblock<Size>
where
    Size: ScreenblockSize,
{
    pub(crate) fn new(size: Size) -> Self {
        let screenblock_ptr = ScreenblockAllocator
            .allocate(size.layout())
            .expect("Not enough space to allocate for background")
            .cast();

        Self {
            ptr: screenblock_ptr,
            size,
        }
    }

    pub(crate) unsafe fn copy_tiles(&self, tiles: &Tiles<Size::TileType>) {
        unsafe {
            self.ptr
                .as_ptr()
                .cast::<Size::TileType>()
                .copy_from_nonoverlapping(tiles.as_ptr(), self.size.num_tiles());
        }
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn screen_base_block(&self) -> u16 {
        let screenblock_location = self.ptr.as_ptr() as usize;
        ((screenblock_location - VRAM_START) / SCREENBLOCK_SIZE) as u16
    }

    pub(crate) fn ptr(&self) -> NonNull<u8> {
        self.ptr
    }
}

impl<Size> Drop for Screenblock<Size>
where
    Size: ScreenblockSize,
{
    fn drop(&mut self) {
        unsafe {
            ScreenblockAllocator.deallocate(self.ptr, self.size.layout());
        }
    }
}

impl ScreenblockSize for RegularBackgroundSize {
    type TileType = Tile;

    fn layout(self) -> Layout {
        RegularBackgroundSize::layout(self)
    }

    fn num_tiles(self) -> usize {
        RegularBackgroundSize::num_tiles(self)
    }
}

impl ScreenblockSize for AffineBackgroundSize {
    type TileType = u8;

    fn layout(self) -> Layout {
        AffineBackgroundSize::layout(self)
    }

    fn num_tiles(self) -> usize {
        AffineBackgroundSize::num_tiles(self)
    }
}
