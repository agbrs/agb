#![warn(missing_docs)]
use core::{alloc::Layout, mem};

use alloc::rc::Rc;

use crate::{
    display::{GraphicsFrame, Priority, tile_data::TileData, tiled::tiles::Tiles},
    fixnum::Vector2D,
};

use super::{
    BackgroundControlRegister, DynamicTile16, RegularBackgroundCommitData, RegularBackgroundData,
    RegularBackgroundId, SCREENBLOCK_SIZE, TRANSPARENT_TILE_INDEX, Tile, TileEffect, TileFormat,
    TileSet, TileSetting, VRAM_MANAGER,
};

pub(crate) use screenblock::RegularBackgroundScreenblock;

use bilge::prelude::*;

mod screenblock;

/// The backgrounds in the GameBoy Advance are made of 8x8 tiles. Each different background option lets
/// you decide how big the background should be before it wraps. Ideally, you should use the smallest background
/// size you can while minimising the number of times you have to redraw tiles.
///
/// If you want more space than can be provided here, or want to keep more video ram free, then you should use
/// the [`InfiniteScrolledMap`](super::InfiniteScrolledMap) which will dynamically load tile data for any size
/// as you scroll around.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum RegularBackgroundSize {
    /// 32x32 tiles (256x256 pixels)
    Background32x32 = 0,
    /// 64x32 tiles (512x256 pixels)
    Background64x32 = 1,
    /// 32x64 tiles (256x512 pixels)
    Background32x64 = 2,
    /// 64x64 tiles (512x512 pixels)
    Background64x64 = 3,
}

impl RegularBackgroundSize {
    const fn width(self) -> usize {
        match self {
            RegularBackgroundSize::Background32x32 => 32,
            RegularBackgroundSize::Background64x32 => 64,
            RegularBackgroundSize::Background32x64 => 32,
            RegularBackgroundSize::Background64x64 => 64,
        }
    }

    const fn height(self) -> usize {
        match self {
            RegularBackgroundSize::Background32x32 => 32,
            RegularBackgroundSize::Background64x32 => 32,
            RegularBackgroundSize::Background32x64 => 64,
            RegularBackgroundSize::Background64x64 => 64,
        }
    }

    const fn size_in_bytes(self) -> usize {
        self.num_tiles() * mem::size_of::<Tile>()
    }

    fn layout(self) -> Layout {
        Layout::from_size_align(self.size_in_bytes(), SCREENBLOCK_SIZE)
            .expect("failed to create layout, should never happen")
    }

    const fn num_tiles(self) -> usize {
        self.width() * self.height()
    }

    const fn gba_offset(self, pos: Vector2D<i32>) -> usize {
        let x_mod = (pos.x & (self.width() as i32 - 1)) as u32;
        let y_mod = (pos.y & (self.height() as i32 - 1)) as u32;

        let screenblock = (x_mod / 32) + (y_mod / 32) * (self.width() as u32 / 32);

        let pos = screenblock * 32 * 32 + (x_mod % 32 + 32 * (y_mod % 32));

        pos as usize
    }
}

/// Represents a collections of background tiles.
///
/// Note that while this is in scope, space in
/// the GBA's VRAM will be allocated and unavailable for other backgrounds. You should use the
/// smallest [`RegularBackgroundSize`] you can while still being able to render the scene you want.
///
/// You can show up to 4 regular backgrounds at once (or 2 regular backgrounds and 1 [affine background](super::AffineBackground)).
///
/// To display a regular background to the screen, you need to call its [`show()`](RegularBackground::show())
/// method on a given [`GraphicsFrame`](crate::display::GraphicsFrame).
///
/// ## Example
///
/// ```rust,no_run
/// # #![no_main]
/// # #![no_std]
/// #
/// # use agb::Gba;
/// # fn foo(gba: &mut Gba) {
/// use agb::display::{
///     Priority,
///     tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
/// };
///
/// let mut gfx = gba.graphics.get();
///
/// let bg = RegularBackground::new(
///     Priority::P0,
///     RegularBackgroundSize::Background32x32,
///     TileFormat::FourBpp
/// );
///
/// // load the background with some tiles
///
/// loop {
///     let mut frame = gfx.frame();
///     bg.show(&mut frame);
///     frame.commit();
/// }
/// # }
/// ```
pub struct RegularBackground {
    priority: Priority,

    tiles: Tiles<Tile>,
    screenblock: Rc<RegularBackgroundScreenblock>,

    scroll: Vector2D<i32>,
}

impl RegularBackground {
    /// Create a new RegularBackground with given `priority`, `size` and `colours`.
    ///
    /// This allocates some space in VRAM to store the actual tile data, but doesn't show anything until you call
    /// the [`show()`](RegularBackground::show()) function on a [`GraphicsFrame`](crate::display::GraphicsFrame).
    ///
    /// You can have more `RegularBackgroundTile` instances then there are backgrounds, but you can only show
    /// 4 at once in a given frame (or 2 and a single [affine background](super::AffineBackground)).
    ///
    /// For [`Priority`], a higher priority is rendered first, so is behind lower priorities. Therefore, `P0`
    /// will be rendered at the _front_ and `P3` at the _back_. For equal priorities, backgrounds are rendered
    /// _behind_ objects.
    #[must_use]
    pub fn new(priority: Priority, size: RegularBackgroundSize, colours: TileFormat) -> Self {
        Self {
            priority,

            tiles: Tiles::new(size.num_tiles(), colours),

            scroll: Vector2D::default(),

            screenblock: Rc::new(RegularBackgroundScreenblock::new(size)),
        }
    }

    /// Sets the scroll position of the background.
    ///
    /// This determines the pixel coordinate of the _screen_
    /// in the background. So increasing the `x` coordinate of the scroll position moves the screen to the right,
    /// effectively rendering the background more to the left.
    ///
    /// To get the current scroll position, you can call [`scroll_pos()`](RegularBackground::scroll_pos()).
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_scroll_pos(&mut self, scroll: impl Into<Vector2D<i32>>) -> &mut Self {
        self.scroll = scroll.into();
        self
    }

    /// Gets the current scroll position of the background.
    ///
    /// This determines the pixel coordinate of the _screen_
    /// in the background. So increasing the `x` coordinate of the scroll position moves the screen to the right,
    /// effectively rendering the background more to the left.
    ///
    /// To set the current scroll position, you can call [`set_scroll_pos()`](RegularBackground::set_scroll_pos()).
    #[must_use]
    pub fn scroll_pos(&self) -> Vector2D<i32> {
        self.scroll
    }

    /// Sets a tile at the given position to the given [`TileSet`] / [`TileSetting`] combination.
    ///
    /// The number of colours which you set when creating the background (in the [`TileFormat`] argument)
    /// must match the number of colours in the tileset you are creating.
    ///
    /// This will resulting in copying the tile data to video RAM. However, setting the same tile across multiple locations
    /// in the background will reference that same tile only once to reduce video RAM usage.
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_tile(
        &mut self,
        pos: impl Into<Vector2D<i32>>,
        tileset: &TileSet<'_>,
        tile_setting: TileSetting,
    ) -> &mut Self {
        assert_eq!(
            tileset.format(),
            self.tiles.colours(),
            "Cannot set a {:?} colour tile on a {:?} colour background",
            tileset.format(),
            self.tiles.colours()
        );

        let pos = self.screenblock.size().gba_offset(pos.into());
        self.set_tile_at_pos(pos, tileset, tile_setting);

        self
    }

    /// Sets a tile at the given position to the given [`DynamicTile16`] / [`TileSetting`] combination.
    ///
    /// This only works on a [16 colour background](TileFormat::FourBpp).
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_tile_dynamic16(
        &mut self,
        pos: impl Into<Vector2D<i32>>,
        tile: &DynamicTile16,
        effect: TileEffect,
    ) -> &mut Self {
        assert_eq!(
            self.tiles.colours(),
            TileFormat::FourBpp,
            "Cannot set a dynamic tile on a {:?} colour background",
            self.tiles.colours()
        );

        let pos = self.screenblock.size().gba_offset(pos.into());
        self.set_tile_at_pos(
            pos,
            &tile.tile_set(),
            TileSetting::new(tile.tile_id(), effect),
        );

        self
    }

    /// Fills the screen with the data given in `tile_data`.
    ///
    /// This is useful mainly e.g. title screens or other full screen backgrounds.
    ///
    /// This method assumes that `tile_data` was loaded via [`include_background_gfx!`](crate::include_background_gfx) and
    /// that it is at least the size of the Game Boy Advance's screen resolution of 240x160 pixels (or 20x30 tiles).
    ///
    /// ## Example
    ///
    /// ```rust
    /// # #![no_main]
    /// # #![no_std]
    /// use agb::{
    ///     display::{
    ///         Priority,
    ///         tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
    ///     },
    ///     include_background_gfx,
    /// };
    ///
    /// include_background_gfx!(mod logo, logo => deduplicate "examples/gfx/test_logo.aseprite");
    ///
    /// # #[agb::doctest]
    /// # fn test(gba: agb::Gba) {
    /// VRAM_MANAGER.set_background_palettes(logo::PALETTES);
    /// let mut bg = RegularBackground::new(Priority::P0, RegularBackgroundSize::Background32x32, TileFormat::FourBpp);
    ///
    /// bg.fill_with(&logo::logo);
    /// # }
    /// ```
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn fill_with(&mut self, tile_data: &TileData) -> &mut Self {
        assert!(
            tile_data.width >= 30,
            "Don't have a full screen's width of tile data, got: {}",
            tile_data.width
        );
        assert!(
            tile_data.height >= 20,
            "Don't have a full screen's height worth of tile data, got: {}",
            tile_data.height
        );
        assert_eq!(
            tile_data.tiles.format(),
            self.tiles.colours(),
            "Cannot set a {:?} colour tile on a {:?} colour background",
            tile_data.tiles.format(),
            self.tiles.colours()
        );

        for y in 0..20 {
            for x in 0..30 {
                let tile_id = y * tile_data.width + x;
                let tile_pos = y * 32 + x;
                self.set_tile_at_pos(tile_pos, &tile_data.tiles, tile_data.tile_settings[tile_id]);
            }
        }

        self
    }

    fn set_tile_at_pos(&mut self, pos: usize, tileset: &TileSet<'_>, tile_setting: TileSetting) {
        let old_tile = self.tiles.get(pos);

        let tile_index = tile_setting.tile_id();

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = VRAM_MANAGER.add_tile(tileset, tile_index, false);
            Tile::new(new_tile_idx, tile_setting)
        } else {
            Tile::default()
        };

        if old_tile != Tile::default() {
            VRAM_MANAGER.remove_tile(old_tile.tile_index(self.tiles.colours()));
        }

        if old_tile == new_tile {
            // no need to mark as dirty if nothing changes
            return;
        }

        self.tiles.set_tile(pos, new_tile);
    }

    /// Show this background on a given frame.
    ///
    /// The background itself won't be visible until you call [`commit()`](GraphicsFrame::commit()) on the provided [`GraphicsFrame`].
    ///
    /// After this call, you can safely drop the background and the provided [`GraphicsFrame`] will maintain the
    /// references needed until the frame is drawn on screen.
    ///
    /// Note that after this call, any modifications made to the background will _not_ show this frame. Effectively
    /// calling `show()` takes a snapshot of the current state of the background, so you can even modify
    /// the background and `show()` it again and both will show in the frame.
    ///
    /// The returned [`RegularBackgroundId`] can be passed to a [`Blend`](crate::display::Blend) or [`Window`](crate::display::Window),
    /// or used for [dma effects](crate::dma).
    ///
    /// # Panics
    ///
    /// If you try to show more than 4 regular backgrounds, or more than 2 backgrounds and a single affine background,
    /// or if there are already 2 affine backgrounds.
    pub fn show(&self, frame: &mut GraphicsFrame<'_>) -> RegularBackgroundId {
        let commit_data = if self.tiles.is_dirty(self.screenblock.ptr()) {
            Some(RegularBackgroundCommitData {
                tiles: self.tiles.clone(),
                screenblock: Rc::clone(&self.screenblock),
            })
        } else {
            None
        };

        frame.bg_frame.set_next_regular(RegularBackgroundData {
            bg_ctrl: self.bg_ctrl_value(),
            scroll_offset: Vector2D::new(self.scroll.x as u16, self.scroll.y as u16),
            commit_data,
        })
    }

    /// Get the size of this background.
    #[must_use]
    pub fn size(&self) -> RegularBackgroundSize {
        self.screenblock.size()
    }

    /// Gets the [`Priority`] of this background.
    #[must_use]
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Sets the [`Priority`] of this background.
    ///
    /// This won't take effect until the next call to [`show()`](RegularBackground::show()).
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = priority;
        self
    }

    fn bg_ctrl_value(&self) -> BackgroundControlRegister {
        let mut background_control_register = BackgroundControlRegister::default();

        background_control_register.set_tile_format(self.tiles.colours().into());
        background_control_register.set_priority(self.priority.into());
        background_control_register
            .set_screen_base_block(u5::new(self.screenblock.screen_base_block() as u8));
        background_control_register.set_screen_size(self.size().into());

        background_control_register
    }
}

#[cfg(test)]
mod test;
