mod infinite_scrolled_map;
mod map;
mod tiled0;
mod tiled1;
mod tiled2;
mod vram_manager;

use crate::bitarray::Bitarray;
use crate::display::Priority;
use agb_fixnum::Vector2D;
use core::cell::RefCell;
pub use infinite_scrolled_map::{InfiniteScrolledMap, PartialUpdateStatus};
pub use map::{AffineMap, MapLoan, RegularMap, TiledMap};
pub use tiled0::Tiled0;
pub use tiled1::Tiled1;
pub use tiled2::Tiled2;
pub use vram_manager::{DynamicTile, TileFormat, TileIndex, TileSet, VRamManager};

use map::TRANSPARENT_TILE_INDEX;

// affine layers start at BG2
pub(crate) const AFFINE_BG_ID_OFFSET: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum RegularBackgroundSize {
    Background32x32 = 0,
    Background64x32 = 1,
    Background32x64 = 2,
    Background64x64 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundID(pub(crate) u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum AffineBackgroundSize {
    Background16x16 = 0,
    Background32x32 = 1,
    Background64x64 = 2,
    Background128x128 = 3,
}

pub trait BackgroundSize {
    #[must_use]
    fn width(&self) -> u32;
    #[must_use]
    fn height(&self) -> u32;
}

pub(super) trait BackgroundSizePrivate: BackgroundSize + Sized {
    fn size_flag(self) -> u16;
    fn num_tiles(&self) -> usize {
        (self.width() * self.height()) as usize
    }
    fn num_screen_blocks(&self) -> usize;
    fn gba_offset(&self, pos: Vector2D<u16>) -> usize;
    fn tile_pos_x(&self, x: i32) -> u16 {
        ((x as u32) & (self.width() - 1)) as u16
    }
    fn tile_pos_y(&self, y: i32) -> u16 {
        ((y as u32) & (self.height() - 1)) as u16
    }
}

impl BackgroundSize for RegularBackgroundSize {
    #[must_use]
    fn width(&self) -> u32 {
        match self {
            RegularBackgroundSize::Background32x64 | RegularBackgroundSize::Background32x32 => 32,
            RegularBackgroundSize::Background64x64 | RegularBackgroundSize::Background64x32 => 64,
        }
    }

    #[must_use]
    fn height(&self) -> u32 {
        match self {
            RegularBackgroundSize::Background32x32 | RegularBackgroundSize::Background64x32 => 32,
            RegularBackgroundSize::Background32x64 | RegularBackgroundSize::Background64x64 => 64,
        }
    }
}

impl BackgroundSizePrivate for RegularBackgroundSize {
    fn size_flag(self) -> u16 {
        self as u16
    }

    fn num_screen_blocks(&self) -> usize {
        self.num_tiles() / (32 * 32)
    }

    // This is hilariously complicated due to how the GBA stores the background screenblocks.
    // See https://www.coranac.com/tonc/text/regbg.htm#sec-map for an explanation
    fn gba_offset(&self, pos: Vector2D<u16>) -> usize {
        let x_mod = pos.x & (self.width() as u16 - 1);
        let y_mod = pos.y & (self.height() as u16 - 1);

        let screenblock = (x_mod / 32) + (y_mod / 32) * (self.width() as u16 / 32);

        let pos = screenblock * 32 * 32 + (x_mod % 32 + 32 * (y_mod % 32));

        pos as usize
    }
}

impl BackgroundSize for AffineBackgroundSize {
    #[must_use]
    fn width(&self) -> u32 {
        match self {
            AffineBackgroundSize::Background16x16 => 16,
            AffineBackgroundSize::Background32x32 => 32,
            AffineBackgroundSize::Background64x64 => 64,
            AffineBackgroundSize::Background128x128 => 128,
        }
    }

    #[must_use]
    fn height(&self) -> u32 {
        self.width()
    }
}

impl BackgroundSizePrivate for AffineBackgroundSize {
    fn size_flag(self) -> u16 {
        self as u16
    }

    fn num_screen_blocks(&self) -> usize {
        // technically 16x16 and 32x32 only use the first 1/8 and 1/2 of the SB, respectively
        1.max(self.num_tiles() / 2048)
    }

    // Affine modes don't do the convoluted staggered block layout
    fn gba_offset(&self, pos: Vector2D<u16>) -> usize {
        let x_mod = pos.x & (self.width() as u16 - 1);
        let y_mod = pos.y & (self.height() as u16 - 1);

        let pos = x_mod + (self.width() as u16 * y_mod);

        pos as usize
    }
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

#[derive(Clone, Copy, Debug, Default)]
pub struct TileSetting(u16);

impl TileSetting {
    pub const BLANK: Self = TileSetting::new(TRANSPARENT_TILE_INDEX, false, false, 0);

    #[must_use]
    pub const fn new(tile_id: u16, hflip: bool, vflip: bool, palette_id: u8) -> Self {
        Self(
            (tile_id & ((1 << 10) - 1))
                | ((hflip as u16) << 10)
                | ((vflip as u16) << 11)
                | ((palette_id as u16) << 12),
        )
    }

    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn hflip(self, should_flip: bool) -> Self {
        Self(self.0 ^ ((should_flip as u16) << 10))
    }

    #[must_use]
    pub const fn vflip(self, should_flip: bool) -> Self {
        Self(self.0 ^ ((should_flip as u16) << 11))
    }

    #[must_use]
    pub const fn palette(self, palette_id: u8) -> Self {
        Self(self.0 ^ ((palette_id as u16) << 12))
    }

    fn index(self) -> u16 {
        self.0 & ((1 << 10) - 1)
    }

    fn setting(self) -> u16 {
        self.0 & !((1 << 10) - 1)
    }
}

fn find_screenblock_gap(screenblocks: &Bitarray<1>, gap: usize) -> usize {
    let mut candidate = 0;

    'outer: while candidate < 16 - gap {
        let starting_point = candidate;
        for attempt in starting_point..(starting_point + gap) {
            if screenblocks.get(attempt) == Some(true) {
                candidate = attempt + 1;
                continue 'outer;
            }
        }

        return candidate;
    }

    panic!(
        "Failed to find screenblock gap of at least {} elements",
        gap
    );
}

trait TiledMode {
    fn screenblocks(&self) -> &RefCell<Bitarray<1>>;
}

trait CreatableRegularTiledMode: TiledMode {
    const REGULAR_BACKGROUNDS: usize;
    fn regular(&self) -> &RefCell<Bitarray<1>>;
}

trait CreatableAffineTiledMode: TiledMode {
    const AFFINE_BACKGROUNDS: usize;
    fn affine(&self) -> &RefCell<Bitarray<1>>;
}

trait RegularTiledMode {
    fn regular_background(
        &self,
        priority: Priority,
        size: RegularBackgroundSize,
        colours: TileFormat,
    ) -> MapLoan<'_, RegularMap>;
}

trait AffineTiledMode {
    fn affine_background(
        &self,
        priority: Priority,
        size: AffineBackgroundSize,
    ) -> MapLoan<'_, AffineMap>;
}

impl<T> RegularTiledMode for T
where
    T: CreatableRegularTiledMode,
{
    fn regular_background(
        &self,
        priority: Priority,
        size: RegularBackgroundSize,
        colours: TileFormat,
    ) -> MapLoan<'_, RegularMap> {
        let mut regular = self.regular().borrow_mut();
        let new_background = regular.first_zero().unwrap();
        if new_background >= T::REGULAR_BACKGROUNDS {
            panic!(
                "can only have {} active regular backgrounds",
                T::REGULAR_BACKGROUNDS
            );
        }

        let num_screenblocks = size.num_screen_blocks();
        let mut screenblocks = self.screenblocks().borrow_mut();

        let screenblock = find_screenblock_gap(&screenblocks, num_screenblocks);
        for id in screenblock..(screenblock + num_screenblocks) {
            screenblocks.set(id, true);
        }

        let bg = RegularMap::new(
            new_background as u8,
            screenblock as u8 + 16,
            priority,
            size,
            colours,
        );

        regular.set(new_background, true);

        MapLoan::new(
            bg,
            new_background as u8,
            screenblock as u8,
            num_screenblocks as u8,
            self.regular(),
            self.screenblocks(),
        )
    }
}

impl<T> AffineTiledMode for T
where
    T: CreatableAffineTiledMode,
{
    fn affine_background(
        &self,
        priority: Priority,
        size: AffineBackgroundSize,
    ) -> MapLoan<'_, AffineMap> {
        let mut affine = self.affine().borrow_mut();
        let new_background = affine.first_zero().unwrap();
        if new_background >= T::AFFINE_BACKGROUNDS + AFFINE_BG_ID_OFFSET {
            panic!(
                "can only have {} active affine backgrounds",
                T::AFFINE_BACKGROUNDS
            );
        }

        let num_screenblocks = size.num_screen_blocks();
        let mut screenblocks = self.screenblocks().borrow_mut();

        let screenblock = find_screenblock_gap(&screenblocks, num_screenblocks);
        for id in screenblock..(screenblock + num_screenblocks) {
            screenblocks.set(id, true);
        }

        let bg = AffineMap::new(new_background as u8, screenblock as u8 + 16, priority, size);

        affine.set(new_background, true);

        MapLoan::new(
            bg,
            new_background as u8,
            screenblock as u8,
            num_screenblocks as u8,
            self.affine(),
            self.screenblocks(),
        )
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

            assert_eq!(size.tile_pos_x(8), 8);
            assert_eq!(size.tile_pos_x(3 + width), 3);
            assert_eq!(size.tile_pos_x(7 + width * 9), 7);

            assert_eq!(size.tile_pos_x(-8), (size.width() - 8) as u16);
            assert_eq!(size.tile_pos_x(-17 - width * 8), (size.width() - 17) as u16);
        }
    }
}
