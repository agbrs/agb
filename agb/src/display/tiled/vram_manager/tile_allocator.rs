use core::{alloc::Layout, ptr::NonNull};

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd},
    display::tiled::{CHARBLOCK_SIZE, VRAM_START},
};

use super::TileFormat;

const fn layout_of(format: TileFormat) -> Layout {
    unsafe { Layout::from_size_align_unchecked(format.tile_size(), format.tile_size()) }
}

const AFFINE_ALLOC_END: usize = VRAM_START + 256 * 8 * 8 / 2;

static AFFINE_TILE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || VRAM_START + 8 * 8,
        end: || AFFINE_ALLOC_END,
    })
};

static TILE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || AFFINE_ALLOC_END,
        end: || VRAM_START + CHARBLOCK_SIZE * 2,
    })
};

#[derive(Clone, Copy, Default)]
pub(crate) struct TileAllocator;

impl TileAllocator {
    pub unsafe fn alloc_for_regular(self, tile_format: TileFormat) -> NonNull<u32> {
        let layout = layout_of(tile_format);

        unsafe {
            match TILE_ALLOCATOR.alloc(layout) {
                Some(ptr) => ptr,
                None => AFFINE_TILE_ALLOCATOR
                    .alloc(layout)
                    .expect("Ran out of video RAM for tiles"),
            }
        }
        .cast()
    }

    pub unsafe fn alloc_for_affine(self) -> NonNull<u32> {
        let layout = layout_of(TileFormat::EightBpp);

        unsafe {
            AFFINE_TILE_ALLOCATOR
                .alloc(layout)
                .expect("Ran out of video RAM for tiles")
        }
        .cast()
    }

    pub unsafe fn dealloc(self, ptr: NonNull<u32>, tile_format: TileFormat) {
        let layout = layout_of(tile_format);

        let allocator = if ptr.addr().get() < AFFINE_ALLOC_END {
            &AFFINE_TILE_ALLOCATOR
        } else {
            &TILE_ALLOCATOR
        };

        unsafe {
            allocator.dealloc(ptr.cast().as_ptr(), layout);
        }
    }
}
