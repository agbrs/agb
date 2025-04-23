mod affine_background;
mod infinite_scrolled_map;
mod registers;
mod regular_background;
mod vram_manager;

use core::marker::PhantomData;

pub use super::affine::AffineMatrixBackground;
use affine_background::AffineBackgroundScreenBlock;
pub use affine_background::{
    AffineBackgroundSize, AffineBackgroundTiles, AffineBackgroundWrapBehaviour,
};
use alloc::rc::Rc;
pub use infinite_scrolled_map::{InfiniteScrolledMap, PartialUpdateStatus};
use regular_background::RegularBackgroundScreenblock;
pub use regular_background::{RegularBackgroundSize, RegularBackgroundTiles};
pub use vram_manager::{DynamicTile16, TileFormat, TileSet, VRAM_MANAGER, VRamManager};

pub(crate) use vram_manager::TileIndex;

pub(crate) use registers::*;

use bilge::prelude::*;

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd, impl_zst_allocator},
    dma::DmaControllable,
    fixnum::{Num, Vector2D},
    memory_mapped::MemoryMapped,
};

use super::DISPLAY_CONTROL;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BackgroundId(pub(crate) u8);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AffineBackgroundId(pub(crate) u8);

impl BackgroundId {
    #[must_use]
    pub fn x_scroll_dma(self) -> DmaControllable<u16> {
        unsafe { DmaControllable::new((0x0400_0010 + self.0 as usize * 4) as *mut _) }
    }
}

const TRANSPARENT_TILE_INDEX: u16 = 0xffff;

#[derive(Clone, Copy, Debug, Default)]
#[repr(align(4))]
pub struct TileSetting {
    tile_id: u16,
    tile_effect: TileEffect,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(transparent)]
pub struct TileEffect(u16);

impl TileSetting {
    /// Displays a blank tile.
    ///
    /// Use this instead of a fully blank tile in your tile set if possible, since it is special cased to be more performant.
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// use agb::{
    ///     display::Priority,
    ///     display::tiled::{
    ///         RegularBackgroundTiles, RegularBackgroundSize, TileEffect, TileSetting,
    ///         VRAM_MANAGER,
    ///     },
    ///     include_background_gfx,
    /// };
    ///
    /// agb::include_background_gfx!(mod water_tiles, tiles => "examples/water_tiles.png");
    ///
    /// # fn foo() {
    /// let mut bg = RegularBackgroundTiles::new(Priority::P0, RegularBackgroundSize::Background32x32, water_tiles::tiles.tiles.format());
    ///
    /// // put something in the background
    /// bg.set_tile((0, 0), &water_tiles::tiles.tiles, water_tiles::tiles.tile_settings[1]);
    /// // set it back to blank
    /// bg.set_tile((0, 0), &water_tiles::tiles.tiles, TileSetting::BLANK);
    /// # }
    /// ```
    pub const BLANK: Self =
        TileSetting::new(TRANSPARENT_TILE_INDEX, TileEffect::new(false, false, 0));

    #[must_use]
    pub const fn new(tile_id: u16, tile_effect: TileEffect) -> Self {
        Self {
            tile_id,
            tile_effect,
        }
    }

    #[must_use]
    pub const fn from_raw(tile_id: u16, effect_bits: u16) -> Self {
        Self {
            tile_id,
            tile_effect: TileEffect(effect_bits),
        }
    }

    pub const fn tile_effect(&mut self) -> &mut TileEffect {
        &mut self.tile_effect
    }

    #[must_use]
    pub const fn hflip(mut self, should_flip: bool) -> Self {
        self.tile_effect().hflip(should_flip);
        self
    }

    #[must_use]
    pub const fn vflip(mut self, should_flip: bool) -> Self {
        self.tile_effect().vflip(should_flip);
        self
    }

    #[must_use]
    pub const fn palette(mut self, palette_id: u8) -> Self {
        self.tile_effect().palette(palette_id);
        self
    }

    #[must_use]
    /// Get the underlying tile id
    pub const fn tile_id(self) -> u16 {
        self.tile_id
    }

    const fn setting(self) -> u16 {
        self.tile_effect.0
    }
}

impl TileEffect {
    #[must_use]
    pub const fn new(hflip: bool, vflip: bool, palette_id: u8) -> Self {
        Self(((hflip as u16) << 10) | ((vflip as u16) << 11) | ((palette_id as u16) << 12))
    }

    pub const fn hflip(&mut self, should_flip: bool) -> &mut Self {
        self.0 ^= (should_flip as u16) << 10;
        self
    }

    pub const fn vflip(&mut self, should_flip: bool) -> &mut Self {
        self.0 ^= (should_flip as u16) << 11;
        self
    }

    pub const fn palette(&mut self, palette_id: u8) -> &mut Self {
        self.0 &= 0x0fff;
        self.0 |= (palette_id as u16) << 12;
        self
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

struct RegularBackgroundCommitData {
    tiles: regular_background::Tiles,
    screenblock: Rc<RegularBackgroundScreenblock>,
}

#[derive(Default)]
struct RegularBackgroundData {
    bg_ctrl: BackgroundControlRegister,
    scroll_offset: Vector2D<u16>,
    commit_data: Option<RegularBackgroundCommitData>,
}

struct AffineBackgroundCommitData {
    tiles: affine_background::Tiles,
    screenblock: Rc<AffineBackgroundScreenBlock>,
}

#[derive(Default)]
struct AffineBackgroundData {
    bg_ctrl: BackgroundControlRegister,
    scroll_offset: Vector2D<Num<i32, 8>>,
    affine_transform: AffineMatrixBackground,
    commit_data: Option<AffineBackgroundCommitData>,
}

pub(crate) struct TiledBackground<'gba> {
    _phantom: PhantomData<&'gba ()>,
}

impl TiledBackground<'_> {
    pub(crate) unsafe fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub(crate) fn iter(&mut self) -> BackgroundFrame<'_> {
        BackgroundFrame {
            _phantom: PhantomData,
            num_regular: 0,
            regular_backgrounds: Default::default(),
            num_affine: 0,
            affine_backgrounds: Default::default(),
        }
    }
}

pub(crate) struct BackgroundFrame<'bg> {
    _phantom: PhantomData<&'bg ()>,

    num_regular: usize,
    regular_backgrounds: [RegularBackgroundData; 4],

    num_affine: usize,
    affine_backgrounds: [AffineBackgroundData; 2],
}

impl BackgroundFrame<'_> {
    fn set_next_regular(&mut self, data: RegularBackgroundData) -> BackgroundId {
        let bg_index = self.next_regular_index();

        self.regular_backgrounds[bg_index] = data;
        BackgroundId(bg_index as u8)
    }

    fn next_regular_index(&mut self) -> usize {
        if self.num_regular + self.num_affine * 2 >= 4 {
            panic!(
                "Can only have 4 backgrounds at once, affine counts as 2. regular: {}, affine: {}",
                self.num_regular, self.num_affine
            );
        }

        let index = self.num_regular;
        self.num_regular += 1;
        index
    }

    fn set_next_affine(&mut self, data: AffineBackgroundData) -> AffineBackgroundId {
        let bg_index = self.next_affine_index();

        self.affine_backgrounds[bg_index - 2] = data;
        AffineBackgroundId(bg_index as u8)
    }

    fn next_affine_index(&mut self) -> usize {
        if self.num_affine * 2 + self.num_regular >= 3 {
            panic!(
                "Can only have 4 backgrounds at once, affine counts as 2. regular: {}, affine: {}",
                self.num_regular, self.num_affine
            );
        }

        let index = self.num_affine;
        self.num_affine += 1;

        index + 2 // first affine BG is bg2
    }

    pub fn commit(mut self) {
        let video_mode = self.num_affine as u16;
        let enabled_backgrounds =
            ((1u16 << self.num_regular) - 1) | (((1 << self.num_affine) - 1) << 2);

        let mut display_control_register = DISPLAY_CONTROL.get();
        display_control_register.set_video_mode(u3::new(video_mode as u8));
        display_control_register.set_enabled_backgrounds(u4::new(enabled_backgrounds as u8));
        display_control_register.set_forced_blank(false);

        DISPLAY_CONTROL.set(display_control_register);

        // It seems weird to put the GC call here, but the `commit_data` could be the last pointer to the
        // actual tile data we want to show, and we want to ensure that all tiles that we're about to print stay alive
        // until the next call to commit.
        VRAM_MANAGER.gc();

        for (i, regular_background) in self
            .regular_backgrounds
            .iter_mut()
            .take(self.num_regular)
            .enumerate()
        {
            let bg_ctrl = unsafe { MemoryMapped::new(0x0400_0008 + i * 2) };
            bg_ctrl.set(regular_background.bg_ctrl);

            let bg_x_offset = unsafe { MemoryMapped::new(0x0400_0010 + i * 4) };
            bg_x_offset.set(regular_background.scroll_offset.x);
            let bg_y_offset = unsafe { MemoryMapped::new(0x0400_0012 + i * 4) };
            bg_y_offset.set(regular_background.scroll_offset.y);

            if let Some(commit_data) = regular_background.commit_data.take() {
                unsafe {
                    commit_data.screenblock.copy_tiles(&commit_data.tiles);
                }
            }
        }

        for (i, affine_background) in self
            .affine_backgrounds
            .iter_mut()
            .take(self.num_affine)
            .enumerate()
        {
            let i = i + 2;

            let bg_ctrl = unsafe { MemoryMapped::new(0x0400_0008 + i * 2) };
            bg_ctrl.set(affine_background.bg_ctrl);

            let bg_x_offset = unsafe { MemoryMapped::new(0x0400_0028 + (i - 2) * 16) };
            bg_x_offset.set(affine_background.scroll_offset.x.to_raw());
            let bg_y_offset = unsafe { MemoryMapped::new(0x0400_002c + (i - 2) * 16) };
            bg_y_offset.set(affine_background.scroll_offset.y.to_raw());

            let affine_transform_offset = unsafe { MemoryMapped::new(0x0400_0020 + (i - 2) * 16) };
            affine_transform_offset.set(affine_background.affine_transform);

            if let Some(commit_data) = affine_background.commit_data.take() {
                unsafe {
                    commit_data.screenblock.copy_tiles(&commit_data.tiles);
                }
            }
        }
    }
}
