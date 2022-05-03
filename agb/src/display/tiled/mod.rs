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

    pub(crate) fn rem_euclid_width(&self, x: i32) -> u16 {
        ((x as u32) & (self.width() - 1)) as u16
    }

    pub(crate) fn rem_euclid_height(&self, y: i32) -> u16 {
        ((y as u32) & (self.height() - 1)) as u16
    }

    pub(crate) fn rem_euclid_width_px(&self, x: i32) -> u16 {
        ((x as u32) & (self.width() * 8 - 1)) as u16
    }

    pub(crate) fn rem_euclid_height_px(&self, y: i32) -> u16 {
        ((y as u32) & (self.height() * 8 - 1)) as u16
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

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn rem_euclid_width_works(_gba: &mut crate::Gba) {
        use RegularBackgroundSize::*;

        let sizes = [
            Background32x32,
            Background32x64,
            Background64x32,
            Background64x64,
        ];

        for size in sizes.iter() {
            let width = size.width() as i32;

            assert_eq!(size.rem_euclid_width(8), 8);
            assert_eq!(size.rem_euclid_width(3 + width), 3);
            assert_eq!(size.rem_euclid_width(7 + width * 9), 7);

            assert_eq!(size.rem_euclid_width(-8), (size.width() - 8) as u16);
            assert_eq!(
                size.rem_euclid_width(-17 - width * 8),
                (size.width() - 17) as u16
            );
        }
    }
}
