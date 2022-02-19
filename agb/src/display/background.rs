use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use alloc::boxed::Box;

pub use super::tiled::VRamManager;
use super::tiled::{RegularMap, TileSetReference, TileSetting};

use crate::bitarray::Bitarray;
use crate::{
    display,
    fixnum::{Rect, Vector2D},
};

use super::{set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, Priority};

pub struct InfiniteScrolledMap<'a> {
    map: MapLoan<'a, RegularMap>,
    get_tile: Box<dyn Fn(Vector2D<i32>) -> (TileSetReference, TileSetting)>,

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
    pub fn new(
        map: MapLoan<'a, RegularMap>,
        get_tile: Box<dyn Fn(Vector2D<i32>) -> (TileSetReference, TileSetting)>,
    ) -> Self {
        Self {
            map,
            get_tile,
            current_pos: (0, 0).into(),
            offset: (0, 0).into(),
            copied_up_to: 0,
        }
    }

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
        let offset_scroll = (
            offset.x.rem_euclid(32 * 8) as u16,
            offset.y.rem_euclid(32 * 8) as u16,
        )
            .into();

        self.map.set_scroll_pos(offset_scroll);
        self.offset = (x_start, y_start).into();

        let copy_from = self.copied_up_to;
        const ROWS_TO_COPY: i32 = 2;

        for (y_idx, y) in
            ((y_start + copy_from)..(y_end.min(y_start + copy_from + ROWS_TO_COPY))).enumerate()
        {
            for (x_idx, x) in (x_start..x_end).enumerate() {
                let pos = (x, y).into();
                let (tile_set_ref, tile_setting) = (self.get_tile)(pos);

                self.map.set_tile(
                    vram,
                    (x_idx as u16, (y_idx + copy_from as usize) as u16).into(),
                    tile_set_ref,
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
            let (tile_set_ref, tile_setting) = (self.get_tile)((tile_x, tile_y).into());

            self.map.set_tile(
                vram,
                (
                    (tile_x - self.offset.x).rem_euclid(32) as u16,
                    (tile_y - self.offset.y).rem_euclid(32) as u16,
                )
                    .into(),
                tile_set_ref,
                tile_setting,
            );
        }

        let current_scroll = self.map.get_scroll_pos();
        let new_scroll = (
            (current_scroll.x as i32 + difference.x).rem_euclid(32 * 8) as u16,
            (current_scroll.y as i32 + difference.y).rem_euclid(32 * 8) as u16,
        )
            .into();

        self.map.set_scroll_pos(new_scroll);

        PartialUpdateStatus::Done
    }

    pub fn show(&mut self) {
        self.map.show();
    }

    pub fn hide(&mut self) {
        self.map.hide();
    }

    pub fn commit(&mut self) {
        self.map.commit();
    }

    pub fn clear(&mut self, vram: &mut VRamManager) {
        self.map.clear(vram);
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

pub struct Tiled0 {
    regular: RefCell<Bitarray<1>>,
}

impl Tiled0 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_settings(GraphicsSettings::empty() | GraphicsSettings::SPRITE1_D);
        set_graphics_mode(DisplayMode::Tiled0);

        Self {
            regular: Default::default(),
        }
    }

    pub fn background(&self, priority: Priority) -> MapLoan<'_, RegularMap> {
        let mut regular = self.regular.borrow_mut();
        let new_background = regular.first_zero().unwrap();
        if new_background >= 4 {
            panic!("can only have 4 active backgrounds");
        }

        let bg = RegularMap::new(new_background as u8, (new_background + 16) as u8, priority);

        regular.set(new_background, true);

        MapLoan::new(bg, new_background as u8, &self.regular)
    }
}

pub struct MapLoan<'a, T> {
    map: T,
    background_id: u8,
    regular_map_list: &'a RefCell<Bitarray<1>>,
}

impl<'a, T> Deref for MapLoan<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'a, T> DerefMut for MapLoan<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<'a, T> MapLoan<'a, T> {
    fn new(map: T, background_id: u8, regular_map_list: &'a RefCell<Bitarray<1>>) -> Self {
        MapLoan {
            map,
            background_id,
            regular_map_list,
        }
    }
}

impl<'a, T> Drop for MapLoan<'a, T> {
    fn drop(&mut self) {
        self.regular_map_list
            .borrow_mut()
            .set(self.background_id as usize, false);
    }
}
