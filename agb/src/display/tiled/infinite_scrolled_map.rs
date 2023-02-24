use alloc::boxed::Box;

use super::{
    BackgroundID, BackgroundSizePrivate, MapLoan, RegularMap, TileSet, TileSetting, TiledMap,
    VRamManager,
};

use crate::{
    display,
    fixnum::{Rect, Vector2D},
};

/// The infinite scrolled map allows you to create a game space larger than a single GBA background.
/// The abstraction allows only for static tiles, but it is possible to animate the tiles if needed.
///
/// When you create a new infinite scrolled map, you need to provide a background which it will render itself
/// onto and a function which takes a `Vector2D<i32>` position and returns which tile should be rendered there.
///
/// The passed function should handle being out of bounds, as the scrolled map does buffer around the edges slightly.
///
/// Note that nothing is copied to video memory until you call [`.commit()`](`InfiniteScrolledMap::commit`), and you
/// must call [`.clear()`](`InfiniteScrolledMap::clear`) before dropping the infinite scrolled map or you will leak video RAM.
///
/// # Example
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// extern crate alloc;
///
/// use alloc::boxed::Box;
///
/// use agb::display::tiled::{
///     InfiniteScrolledMap,
///     TileSetting,
///     RegularBackgroundSize,
///     TileSet,
///     TileFormat,
/// };
/// use agb::display::Priority;
///
/// mod tilemap {
///    pub const BACKGROUND_MAP: &[u16] = &[ // Probably load this from a file
/// # 0, 1, 2];
///    pub const WIDTH: i32 = // set it to some width
/// # 12;
///    pub const MAP_TILES: &[u8] = &[ // probably load this from a file
/// # 0];
/// }
///
/// # fn foo(mut gba: agb::Gba) {
/// let (gfx, mut vram) = gba.display.video.tiled0();
///
/// let tileset = TileSet::new(&tilemap::MAP_TILES, TileFormat::FourBpp);
///
/// let mut backdrop = InfiniteScrolledMap::new(
///     gfx.background(Priority::P2, RegularBackgroundSize::Background32x32, TileFormat::FourBpp),
///     Box::new(|pos| {
///         (
///             &tileset,
///             TileSetting::from_raw(
///                 *tilemap::BACKGROUND_MAP
///                     .get((pos.x + tilemap::WIDTH * pos.y) as usize)
///                     .unwrap_or(&0),
///             ),
///         )
///     }),
/// );
///
/// // ...
///
/// backdrop.set_pos(&mut vram, (3, 5).into());
/// backdrop.commit(&mut vram);
/// backdrop.show();
/// # }
/// ```
pub struct InfiniteScrolledMap<'a> {
    map: MapLoan<'a, RegularMap>,
    tile: Box<dyn Fn(Vector2D<i32>) -> (&'a TileSet<'a>, TileSetting) + 'a>,

    current_pos: Vector2D<i32>,
    offset: Vector2D<i32>,

    copied_up_to: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialUpdateStatus {
    Done,
    Continue,
}

impl<'a> InfiniteScrolledMap<'a> {
    /// Creates a new infinite scrolled map wrapping the provided background using the given function to
    /// position tiles.
    ///
    /// This will not actually render anything until either [`.init()`](`InfiniteScrolledMap::init`) or
    /// [`.init_partial()`](`InfiniteScrolledMap::init_partial`) is called to set up VRam and this is then
    /// [`committed`](`InfiniteScrolledMap::commit`).
    #[must_use]
    pub fn new(
        map: MapLoan<'a, RegularMap>,
        tile: Box<dyn Fn(Vector2D<i32>) -> (&'a TileSet<'a>, TileSetting) + 'a>,
    ) -> Self {
        Self {
            map,
            tile,
            current_pos: (0, 0).into(),
            offset: (0, 0).into(),
            copied_up_to: 0,
        }
    }

    /// Initialises the map and fills it, calling the between_updates occasionally to allow you to ensure that
    /// music keeps playing without interruption.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # #![no_std]
    /// # #![no_main]
    /// # extern crate alloc;
    /// #
    /// # use alloc::boxed::Box;
    /// #
    /// # use agb::display::tiled::{
    /// #    InfiniteScrolledMap,
    /// #    TileSetting,
    /// #    RegularBackgroundSize,
    /// #    TileSet,
    /// #    TileFormat,
    /// # };
    /// # use agb::display::Priority;
    /// #
    /// # mod tilemap {
    /// #   pub const BACKGROUND_MAP: &[u16] = &[0, 1, 2];
    /// #   pub const WIDTH: i32 = 12;
    /// #   pub const MAP_TILES: &[u8] = &[0];
    /// # }
    /// #
    /// # fn foo(mut gba: agb::Gba) {
    /// # let (gfx, mut vram) = gba.display.video.tiled0();
    /// #
    /// # let tileset = TileSet::new(&tilemap::MAP_TILES, TileFormat::FourBpp);
    /// #
    /// # let mut backdrop = InfiniteScrolledMap::new(
    /// #    gfx.background(Priority::P2, RegularBackgroundSize::Background32x32, TileFormat::FourBpp),
    /// #    Box::new(|pos| {
    /// #        (
    /// #            &tileset,
    /// #            TileSetting::from_raw(
    /// #                *tilemap::BACKGROUND_MAP
    /// #                    .get((pos.x + tilemap::WIDTH * pos.y) as usize)
    /// #                     .unwrap_or(&0),
    /// #            ),
    /// #        )
    /// #    }),
    /// # );
    /// #
    /// # let vblank = agb::interrupt::VBlank::get();
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// let start_position = agb::fixnum::Vector2D::new(10, 10);
    /// backdrop.init(&mut vram, start_position, &mut || {
    ///     vblank.wait_for_vblank();
    ///     mixer.frame();
    /// });
    /// # }
    /// ```
    pub fn init(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<i32>,
        between_updates: &mut impl FnMut(),
    ) {
        while self.init_partial(vram, pos) != PartialUpdateStatus::Done {
            between_updates();
        }
    }

    /// Does a partial initialisation of the background, rendering 2 rows.
    /// This is because initialisation can take quite a while, so you will need to call
    /// this method a few times to ensure that you update the entire frame.
    ///
    /// Returns [`PartialUpdateStatus::Done`] if complete, and [`PartialUpdateStatus::Continue`]
    /// if you need to call this a few more times to fully update the screen.
    ///
    /// It is recommended you use [`.init()`](`InfiniteScrolledMap::init`) instead of this method
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # #![no_std]
    /// # #![no_main]
    /// # extern crate alloc;
    /// #
    /// # use alloc::boxed::Box;
    /// #
    /// # use agb::display::tiled::{
    /// #    InfiniteScrolledMap,
    /// #    TileSetting,
    /// #    RegularBackgroundSize,
    /// #    TileSet,
    /// #    TileFormat,
    /// #    PartialUpdateStatus,
    /// # };
    /// # use agb::display::Priority;
    /// #
    /// # mod tilemap {
    /// #   pub const BACKGROUND_MAP: &[u16] = &[0, 1, 2];
    /// #   pub const WIDTH: i32 = 12;
    /// #   pub const MAP_TILES: &[u8] = &[0];
    /// # }
    /// #
    /// # fn foo(mut gba: agb::Gba) {
    /// # let (gfx, mut vram) = gba.display.video.tiled0();
    /// #
    /// # let tileset = TileSet::new(&tilemap::MAP_TILES, TileFormat::FourBpp);
    /// #
    /// # let mut backdrop = InfiniteScrolledMap::new(
    /// #    gfx.background(Priority::P2, RegularBackgroundSize::Background32x32, TileFormat::FourBpp),
    /// #    Box::new(|pos| {
    /// #        (
    /// #            &tileset,
    /// #            TileSetting::from_raw(
    /// #                *tilemap::BACKGROUND_MAP
    /// #                    .get((pos.x + tilemap::WIDTH * pos.y) as usize)
    /// #                     .unwrap_or(&0),
    /// #            ),
    /// #        )
    /// #    }),
    /// # );
    /// #
    /// # let vblank = agb::interrupt::VBlank::get();
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// let start_position = agb::fixnum::Vector2D::new(10, 10);
    /// while backdrop.init_partial(&mut vram, start_position) == PartialUpdateStatus::Continue {
    ///     vblank.wait_for_vblank();
    ///     mixer.frame();
    /// }
    /// # }
    /// ```
    pub fn init_partial(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<i32>,
    ) -> PartialUpdateStatus {
        self.current_pos = pos;

        let x_start = div_floor(self.current_pos.x, 8);
        let y_start = div_floor(self.current_pos.y, 8);

        let x_end = div_ceil(self.current_pos.x + display::WIDTH, 8) + 1;
        let y_end = div_ceil(self.current_pos.y + display::HEIGHT, 8) + 1;

        let offset = self.current_pos - (x_start * 8, y_start * 8).into();

        self.map
            .set_scroll_pos((offset.x as i16, offset.y as i16).into());
        self.offset = (x_start, y_start).into();

        let copy_from = self.copied_up_to;
        const ROWS_TO_COPY: i32 = 2;

        for (y_idx, y) in
            ((y_start + copy_from)..(y_end.min(y_start + copy_from + ROWS_TO_COPY))).enumerate()
        {
            for (x_idx, x) in (x_start..x_end).enumerate() {
                let pos = (x, y).into();
                let (tileset, tile_setting) = (self.tile)(pos);

                self.map.set_tile(
                    vram,
                    (x_idx as u16, (y_idx + copy_from as usize) as u16).into(),
                    tileset,
                    tile_setting,
                );
            }
        }

        if copy_from + ROWS_TO_COPY >= y_end - y_start {
            self.copied_up_to = 0;
            PartialUpdateStatus::Done
        } else {
            self.copied_up_to = copy_from + ROWS_TO_COPY;
            PartialUpdateStatus::Continue
        }
    }

    /// Set the top left corner of the map. You may need to call this method multiple times if
    /// [`PartialUpdateStatus::Continue`] is returned.
    pub fn set_pos(
        &mut self,
        vram: &mut VRamManager,
        new_pos: Vector2D<i32>,
    ) -> PartialUpdateStatus {
        let old_pos = self.current_pos;

        let difference = new_pos - old_pos;

        if difference.x.abs() > 10 * 8 || difference.y.abs() > 10 * 8 {
            return self.init_partial(vram, new_pos);
        }

        self.current_pos = new_pos;

        let new_tile_x = div_floor(new_pos.x, 8);
        let new_tile_y = div_floor(new_pos.y, 8);

        let difference_tile_x = div_ceil(difference.x, 8);
        let difference_tile_y = div_ceil(difference.y, 8);

        let size = self.map.size();

        let vertical_rect_to_update: Rect<i32> = if div_floor(old_pos.x, 8) != new_tile_x {
            // need to update the x line
            // calculate which direction we need to update
            let direction = difference.x.signum();

            // either need to update 20 or 21 tiles depending on whether the y coordinate is a perfect multiple
            let y_tiles_to_update = 22;

            let line_to_update = if direction < 0 {
                // moving to the left, so need to update the left most position
                new_tile_x
            } else {
                // moving to the right, so need to update the right most position
                new_tile_x + 30 // TODO is this correct?
            };

            Rect::new(
                (line_to_update, new_tile_y - 1).into(),
                (difference_tile_x, y_tiles_to_update).into(),
            )
        } else {
            Rect::new((0i32, 0).into(), (0i32, 0).into())
        };

        let horizontal_rect_to_update: Rect<i32> = if div_floor(old_pos.y, 8) != new_tile_y {
            // need to update the y line
            // calculate which direction we need to update
            let direction = difference.y.signum();

            // either need to update 30 or 31 tiles depending on whether the x coordinate is a perfect multiple
            let x_tiles_to_update: i32 = 32;

            let line_to_update = if direction < 0 {
                // moving up so need to update the top
                new_tile_y
            } else {
                // moving down so need to update the bottom
                new_tile_y + 20 // TODO is this correct?
            };

            Rect::new(
                (new_tile_x - 1, line_to_update).into(),
                (x_tiles_to_update, difference_tile_y).into(),
            )
        } else {
            Rect::new((0i32, 0).into(), (0i32, 0).into())
        };

        for (tile_x, tile_y) in vertical_rect_to_update
            .iter()
            .chain(horizontal_rect_to_update.iter())
        {
            let (tileset, tile_setting) = (self.tile)((tile_x, tile_y).into());

            self.map.set_tile(
                vram,
                (
                    size.tile_pos_x(tile_x - self.offset.x),
                    size.tile_pos_y(tile_y - self.offset.y),
                )
                    .into(),
                tileset,
                tile_setting,
            );
        }

        let current_scroll = self.map.scroll_pos();
        let new_scroll = current_scroll + (difference.x as i16, difference.y as i16).into();

        self.map.set_scroll_pos(new_scroll);

        PartialUpdateStatus::Done
    }

    /// Makes the map visible
    pub fn show(&mut self) {
        self.map.show();
    }

    /// Hides the map
    pub fn hide(&mut self) {
        self.map.hide();
    }

    /// Copies data to vram. Needs to be called during vblank if possible
    pub fn commit(&mut self, vram: &mut VRamManager) {
        self.map.commit(vram);
    }

    /// Clears the underlying map. You must call this before the scrolled map goes out of scope
    /// or you will leak VRam.
    pub fn clear(&mut self, vram: &mut VRamManager) {
        self.map.clear(vram);
    }

    #[must_use]
    pub const fn background(&self) -> BackgroundID {
        self.map.background()
    }
}

fn div_floor(x: i32, y: i32) -> i32 {
    if x > 0 && y < 0 {
        (x - 1) / y - 1
    } else if x < 0 && y > 0 {
        (x + 1) / y - 1
    } else {
        x / y
    }
}

fn div_ceil(x: i32, y: i32) -> i32 {
    if x > 0 && y > 0 {
        (x - 1) / y + 1
    } else if x < 0 && y < 0 {
        (x + 1) / y + 1
    } else {
        x / y
    }
}
