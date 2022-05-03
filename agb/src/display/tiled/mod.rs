mod infinite_scrolled_map;
mod map;
mod tiled0;
mod vram_manager;

pub use infinite_scrolled_map::{InfiniteScrolledMap, PartialUpdateStatus};
pub use map::{MapLoan, RegularMap};
pub use tiled0::Tiled0;
pub use vram_manager::{DynamicTile, TileFormat, TileIndex, TileSet, VRamManager};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegularBackgroundSize {
    Background32x32,
    Background64x32,
    Background32x64,
    Background64x64,
}

impl RegularBackgroundSize {
    pub fn width(&self) -> u32 {
        match self {
            RegularBackgroundSize::Background32x32 => 32,
            RegularBackgroundSize::Background64x32 => 64,
            RegularBackgroundSize::Background32x64 => 32,
            RegularBackgroundSize::Background64x64 => 64,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            RegularBackgroundSize::Background32x32 => 32,
            RegularBackgroundSize::Background64x32 => 32,
            RegularBackgroundSize::Background32x64 => 64,
            RegularBackgroundSize::Background64x64 => 64,
        }
    }

    pub(crate) fn size_flag(&self) -> u16 {
        match self {
            RegularBackgroundSize::Background32x32 => 0,
            RegularBackgroundSize::Background64x32 => 1,
            RegularBackgroundSize::Background32x64 => 2,
            RegularBackgroundSize::Background64x64 => 3,
        }
    }

    pub(crate) fn num_tiles(&self) -> usize {
        (self.width() * self.height()) as usize
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
struct Tile(u16);

impl Tile {
    fn new(idx: TileIndex, setting: TileSetting) -> Self {
        Self(idx.index() | setting.setting())
    }

    fn tile_index(self) -> TileIndex {
        TileIndex::new(self.0 as usize & ((1 << 10) - 1))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TileSetting(u16);

impl TileSetting {
    pub const fn new(tile_id: u16, hflip: bool, vflip: bool, palette_id: u8) -> Self {
        Self(
            (tile_id & ((1 << 10) - 1))
                | ((hflip as u16) << 10)
                | ((vflip as u16) << 11)
                | ((palette_id as u16) << 12),
        )
    }

    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    fn index(self) -> u16 {
        self.0 & ((1 << 10) - 1)
    }

    fn setting(self) -> u16 {
        self.0 & !((1 << 10) - 1)
    }
}
