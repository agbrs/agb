mod regular_background;
mod vram_manager;

use core::{cell::RefCell, marker::PhantomData};

pub use regular_background::{RegularBackgroundSize, RegularBackgroundTiles};
pub use vram_manager::{DynamicTile, TileFormat, TileIndex, TileSet, VRamManager};

use crate::agb_alloc::{
    block_allocator::BlockAllocator, bump_allocator::StartEnd, impl_zst_allocator,
};

pub struct BackgroundId(pub(crate) u8);

const TRANSPARENT_TILE_INDEX: u16 = 0xffff;

#[derive(Clone, Copy, Debug, Default)]
pub struct TileSetting {
    tile_id: u16,
    effect_bits: u16,
}

impl TileSetting {
    pub const BLANK: Self = TileSetting::new(TRANSPARENT_TILE_INDEX, false, false, 0);

    #[must_use]
    pub const fn new(tile_id: u16, hflip: bool, vflip: bool, palette_id: u8) -> Self {
        Self {
            tile_id,
            effect_bits: ((hflip as u16) << 10)
                | ((vflip as u16) << 11)
                | ((palette_id as u16) << 12),
        }
    }

    #[must_use]
    pub const fn hflip(self, should_flip: bool) -> Self {
        Self {
            effect_bits: self.effect_bits ^ ((should_flip as u16) << 10),
            ..self
        }
    }

    #[must_use]
    pub const fn vflip(self, should_flip: bool) -> Self {
        Self {
            effect_bits: self.effect_bits ^ ((should_flip as u16) << 11),
            ..self
        }
    }

    #[must_use]
    pub const fn palette(self, palette_id: u8) -> Self {
        Self {
            effect_bits: self.effect_bits ^ ((palette_id as u16) << 12),
            ..self
        }
    }

    fn index(self) -> u16 {
        self.tile_id
    }

    fn setting(self) -> u16 {
        self.effect_bits
    }
}

struct TiledBackgroundModifyables {}

pub struct TiledBackground<'gba> {
    _phantom: PhantomData<&'gba ()>,
    frame_data: RefCell<TiledBackgroundModifyables>,
}

pub struct BackgroundIterator<'bg> {
    frame_data: &'bg RefCell<TiledBackgroundModifyables>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
struct Tile(u16);

impl Tile {
    fn new(idx: TileIndex, setting: TileSetting) -> Self {
        Self(idx.raw_index() | setting.setting())
    }

    fn tile_index(self, format: TileFormat) -> TileIndex {
        TileIndex::new(self.0 as usize & ((1 << 10) - 1), format)
    }
}

struct ScreenblockAllocator;

pub(crate) const VRAM_START: usize = 0x0600_0000;
pub(crate) const SCREENBLOCK_SIZE: usize = 0x800;
pub(crate) const CHARBLOCK_SIZE: usize = SCREENBLOCK_SIZE * 8;

const SCREENBLOCK_ALLOC_START: usize = VRAM_START + CHARBLOCK_SIZE * 2;

static SCREENBLOCK_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || SCREENBLOCK_ALLOC_START,
        end: || SCREENBLOCK_ALLOC_START + 0x4000,
    })
};

impl_zst_allocator!(ScreenblockAllocator, SCREENBLOCK_ALLOCATOR);
