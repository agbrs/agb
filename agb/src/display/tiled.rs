//! Anything to do with tiled backgrounds
//!
//! Most games made for the Game Boy Advance use tiled backgrounds.
//! You can create and manage regular backgrounds using the [`RegularBackground`] struct,
//! and affine backgrounds using the [`AffineBackground`] struct.
//!
//! Palettes are managed using the [`VRAM_MANAGER`] global value.
//!
//! See the [background deep dive](https://agbrs.dev/book/articles/backgrounds.html) for further details about backgrounds.
#![warn(missing_docs)]
mod affine_background;
mod infinite_scrolled_map;
mod registers;
mod regular_background;
mod vram_manager;

use core::marker::PhantomData;

use affine_background::AffineBackgroundScreenBlock;
pub use affine_background::{
    AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour, AffineMatrixBackground,
};
use alloc::rc::Rc;
pub use infinite_scrolled_map::{InfiniteScrolledMap, PartialUpdateStatus};
use regular_background::RegularBackgroundScreenblock;
pub use regular_background::{RegularBackground, RegularBackgroundSize};
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

/// Represents a background which can be either [regular](RegularBackground) or [affine](AffineBackground).
///
/// You never need to create this directly, instead using the `From` implementation from [`AffineBackgroundId`] or
/// [`RegularBackgroundId`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BackgroundId(pub(crate) u8);

impl From<RegularBackgroundId> for BackgroundId {
    fn from(value: RegularBackgroundId) -> Self {
        Self(value.0)
    }
}

impl From<AffineBackgroundId> for BackgroundId {
    fn from(value: AffineBackgroundId) -> Self {
        Self(value.0)
    }
}

/// Represents a [regular background](RegularBackground) that's about to be displayed.
///
/// This is returned by the [`show()`](RegularBackground::show) method. You'll need this if you want
/// to apply additional effects on the background while it is being displayed, such as adding it to a
/// [`Window`](super::Window::enable_background) or using one of the DMA registers.
///
/// See the `dma_effect_background_*` [examples](https://agbrs.dev/examples) for examples of how to use the DMA functions.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RegularBackgroundId(pub(crate) u8);

impl RegularBackgroundId {
    /// Control the x scroll position every scan line
    #[must_use]
    pub fn x_scroll_dma(self) -> DmaControllable<u16> {
        unsafe { DmaControllable::new((0x0400_0010 + self.0 as usize * 4) as *mut _) }
    }

    /// Control the y scroll position every scan line
    #[must_use]
    pub fn y_scroll_dma(self) -> DmaControllable<u16> {
        unsafe { DmaControllable::new((0x0400_0012 + self.0 as usize * 4) as *mut _) }
    }

    /// Control the current scroll position every scan line
    #[must_use]
    pub fn scroll_dma(self) -> DmaControllable<Vector2D<u16>> {
        unsafe { DmaControllable::new((0x0400_0010 + self.0 as usize * 4) as *mut _) }
    }
}

/// Represents an [affine background](AffineBackground) that's about to be displayed.
///
/// This is returned by the [`show()`](AffineBackground::show) method. You'll need this if you
/// want to apply additional effects such as using it with DMA.
///
/// See the `dma_effect_affine_background_*` [examples](https://agbrs.dev/examples) for examples of how to use the DMA transform function.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AffineBackgroundId(pub(crate) u8);

impl AffineBackgroundId {
    /// Change the transformation matrix every scan line
    ///
    /// Note that the current scroll position resets if you change the background transform as part of the DMA
    /// (unlike the regular background DMA), so you may need to add the current `y` offset to the position if you
    /// are looking for it to be displayed normally.
    #[must_use]
    pub fn transform_dma(self) -> DmaControllable<AffineMatrixBackground> {
        unsafe { DmaControllable::new((0x0400_0020 + (self.0 as usize - 2) * 16) as *mut _) }
    }
}

const TRANSPARENT_TILE_INDEX: u16 = 0xffff;

/// The `TileSetting` holds the index for the tile in the tile set, and which effects it should be rendered with.
///
/// You will mainly get a TileSetting from [`TileData.tile_settings`](super::tile_data::TileData::tile_settings) which
/// is produced by the [`include_background_gfx!`](crate::include_background_gfx) macro.
#[derive(Clone, Copy, Debug, Default)]
#[repr(align(4))]
pub struct TileSetting {
    tile_id: u16,
    tile_effect: TileEffect,
}

/// Represents the simple effects that can be applied to a tile.
///
/// A tile can be flipped horizontally, vertically or both. You can also configure which
/// palette to use for the tile.
///
/// The palette does nothing for 256 colour tiles, since there is only a single 256 colour palette.
#[derive(Clone, Copy, Debug, Default)]
#[repr(transparent)]
pub struct TileEffect(u16);

impl TileSetting {
    /// Displays a blank tile.
    ///
    /// Use this instead of a fully blank tile in your tile set if possible, since it is special cased to be more performant.
    ///
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// # core::include!("../doctest_runner.rs");
    /// use agb::{
    ///     display::Priority,
    ///     display::tiled::{
    ///         RegularBackground, RegularBackgroundSize, TileEffect, TileSetting,
    ///         VRAM_MANAGER,
    ///     },
    ///     include_background_gfx,
    /// };
    ///
    /// agb::include_background_gfx!(mod water_tiles, tiles => "examples/water_tiles.png");
    ///
    /// # fn test(gba: agb::Gba) {
    /// let mut bg = RegularBackground::new(Priority::P0, RegularBackgroundSize::Background32x32, water_tiles::tiles.tiles.format());
    ///
    /// // put something in the background
    /// bg.set_tile((0, 0), &water_tiles::tiles.tiles, water_tiles::tiles.tile_settings[1]);
    /// // set it back to blank
    /// bg.set_tile((0, 0), &water_tiles::tiles.tiles, TileSetting::BLANK);
    /// # }
    /// ```
    pub const BLANK: Self =
        TileSetting::new(TRANSPARENT_TILE_INDEX, TileEffect::new(false, false, 0));

    /// Create a new TileIndex with a given `tile_id` and `tile_effect`.
    ///
    /// You probably won't need to use this method, instead either using [`TileSetting::BLANK`] or one of the entries in
    /// [`TileData.tile_settings`](crate::display::tile_data::TileData::tile_settings).
    #[must_use]
    pub const fn new(tile_id: u16, tile_effect: TileEffect) -> Self {
        Self {
            tile_id,
            tile_effect,
        }
    }

    /// Gets the tile_effect and allows for manipulations of it.
    pub const fn tile_effect(&mut self) -> &mut TileEffect {
        &mut self.tile_effect
    }

    /// Horizontally flips the tile.
    ///
    /// If `should_flip` is false, returns the same as current. If it is true, will return a new
    /// `TileSetting` with ever setting the same, except it will be horizontally flipped.
    #[must_use]
    pub const fn hflip(mut self, should_flip: bool) -> Self {
        self.tile_effect().hflip(should_flip);
        self
    }

    /// Vertically flips the tile.
    ///
    /// If `should_flip` is false, returns the same as current. If it is true, will return a new
    /// `TileSetting` with ever setting the same, except it will be vertically flipped.
    #[must_use]
    pub const fn vflip(mut self, should_flip: bool) -> Self {
        self.tile_effect().vflip(should_flip);
        self
    }

    /// Sets which palette to use
    ///
    /// This has no effect if the background is set to use 256 colours.
    #[must_use]
    pub const fn palette(mut self, palette_id: u8) -> Self {
        self.tile_effect().palette(palette_id);
        self
    }

    /// Gets the internal tile ID for a given tile.
    ///
    /// The main use case for this is checking which tile_id was assigned when using the `deduplicate`
    /// option in [`include_background_gfx!()`](crate::include_background_gfx).
    ///
    /// Be careful when passing this ID to [`VRAM_MANAGER.replace_tile()`](crate::display::tiled::VRamManager::replace_tile)
    /// if you've generated this tile set with the `deduplicate` option, since tiles may be flipped or
    /// reused meaning replacing IDs could result in strange display behaviour.
    #[must_use]
    pub const fn tile_id(self) -> u16 {
        self.tile_id
    }

    const fn setting(self) -> u16 {
        self.tile_effect.0
    }
}

impl TileEffect {
    /// Creates a new [`TileEffect`] with the given state of being flipped and palette id.
    #[must_use]
    pub const fn new(hflip: bool, vflip: bool, palette_id: u8) -> Self {
        Self(((hflip as u16) << 10) | ((vflip as u16) << 11) | ((palette_id as u16) << 12))
    }

    /// Horizontally flips the tile.
    ///
    /// If `should_flip` is false, this does nothing.
    /// If `should_flip` is true, will mutate itself to show the tile flipped horizontally.
    ///
    /// Calling `.hflip` twice on the same TileEffect will flip the tile twice, resulting in no flipping.
    pub const fn hflip(&mut self, should_flip: bool) -> &mut Self {
        self.0 ^= (should_flip as u16) << 10;
        self
    }

    /// Vertically flips the tile.
    ///
    /// If `should_flip` is false, this does nothing.
    /// If `should_flip` is true, will mutate itself to show the tile flipped vertically.
    ///
    /// Calling `.hflip` twice on the same TileEffect will flip the tile twice, resulting in no flipping.
    pub const fn vflip(&mut self, should_flip: bool) -> &mut Self {
        self.0 ^= (should_flip as u16) << 11;
        self
    }

    /// Sets the palette index for the current TileEffect.
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
    fn set_next_regular(&mut self, data: RegularBackgroundData) -> RegularBackgroundId {
        let bg_index = self.next_regular_index();

        self.regular_backgrounds[bg_index] = data;
        RegularBackgroundId(bg_index as u8)
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
