use alloc::boxed::Box;

use super::{MapLoan, RegularMap, TileSet, TileSetting, VRamManager};

use crate::{
    display,
    fixnum::{Rect, Vector2D},
};

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
            self.map.size().rem_euclid_width(offset.x) as u16,
            self.map.size().rem_euclid_height(offset.y) as u16,
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
                    size.rem_euclid_width(tile_x - self.offset.x) as u16,
                    size.rem_euclid_height(tile_y - self.offset.y) as u16,
                )
                    .into(),
                tileset,
                tile_setting,
            );
        }

        let current_scroll = self.map.scroll_pos();
        let new_scroll = (
            size.rem_euclid_width_px(current_scroll.x as i32 + difference.x) as u16,
            size.rem_euclid_height_px(current_scroll.y as i32 + difference.y) as u16,
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

    pub fn commit(&mut self, vram: &mut VRamManager) {
        self.map.commit(vram);
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
