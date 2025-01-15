use agb_fixnum::{vec2, Rect, Vector2D};
use alloc::boxed::Box;

use crate::display;

use super::{
    BackgroundId, BackgroundIterator, RegularBackgroundTiles, TileSet, TileSetting, VRamManager,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialUpdateStatus {
    Done,
    Continue,
}

pub struct InfiniteScrolledMap {
    map: RegularBackgroundTiles,
    tile: Box<dyn Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting)>,

    current_pos: Vector2D<i32>,
    offset: Vector2D<i32>,

    copied_up_to: i32,
}

impl InfiniteScrolledMap {
    #[must_use]
    pub fn new(
        map: RegularBackgroundTiles,
        tile: Box<dyn Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting)>,
    ) -> Self {
        Self {
            map,
            tile,
            current_pos: (0, 0).into(),
            offset: (0, 0).into(),
            copied_up_to: 0,
        }
    }

    pub fn init_partial(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<i32>,
    ) -> PartialUpdateStatus {
        self.current_pos = pos;

        let x_start = self.current_pos.x.div_floor_stable(8);
        let y_start = self.current_pos.y.div_floor_stable(8);

        let x_end = (self.current_pos.x + display::WIDTH).div_ceil_stable(8) + 1;
        let y_end = (self.current_pos.y + display::HEIGHT).div_ceil_stable(8) + 1;

        let offset = self.current_pos - vec2(x_start, y_start) * 8;

        self.map.set_scroll_pos((offset.x as i16, offset.y as i16));
        self.offset = vec2(x_start, y_start);

        let copy_from = self.copied_up_to;
        const ROWS_TO_COPY: i32 = 2;

        for (y_idx, y) in
            ((y_start + copy_from)..(y_end.min(y_start + copy_from + ROWS_TO_COPY))).enumerate()
        {
            for (x_idx, x) in (x_start..x_end).enumerate() {
                let pos = vec2(x, y);
                let (tileset, tile_setting) = (self.tile)(pos);

                self.map.set_tile(
                    vram,
                    (x_idx as u16, (y_idx + copy_from as usize) as u16),
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
        self.current_pos = new_pos;

        let old_tile_x = old_pos.x.div_floor_stable(8);
        let old_tile_y = old_pos.y.div_floor_stable(8);

        let new_tile_x = new_pos.y.div_floor_stable(8);
        let new_tile_y = new_pos.y.div_floor_stable(8);

        let difference_tile_x = new_tile_x - old_tile_x;
        let difference_tile_y = new_tile_y - old_tile_y;

        let size = self.map.size();

        let vertical_rect_to_update: Rect<i32> = if difference_tile_x != 0 {
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
                new_tile_x + 31 // TODO is this correct?
            };

            Rect::new(
                (line_to_update, new_tile_y - 1).into(),
                (-difference_tile_x, y_tiles_to_update).into(),
            )
            .abs()
        } else {
            Rect::new(vec2(0, 0), vec2(0, 0))
        };

        let horizontal_rect_to_update: Rect<i32> = if difference_tile_y != 0 {
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
                new_tile_y + 21 // TODO is this correct?
            };

            Rect::new(
                (new_tile_x - 1, line_to_update).into(),
                (x_tiles_to_update, -difference_tile_y).into(),
            )
            .abs()
        } else {
            Rect::new((0i32, 0).into(), (0i32, 0).into())
        };

        for (tile_x, tile_y) in vertical_rect_to_update
            .iter()
            .chain(horizontal_rect_to_update.iter())
        {
            let (tileset, tile_setting) = (self.tile)(vec2(tile_x, tile_y));

            self.map.set_tile(
                vram,
                size.tile_pos(vec2(tile_x, tile_y) - self.offset),
                tileset,
                tile_setting,
            );
        }

        let current_scroll = self.map.scroll_pos();
        let new_scroll = current_scroll + (difference.x as i16, difference.y as i16).into();

        self.map.set_scroll_pos(new_scroll);

        PartialUpdateStatus::Done
    }

    pub fn commit(&mut self) {
        self.map.commit();
    }

    pub fn show(&self, bg_iter: &mut BackgroundIterator<'_>) -> BackgroundId {
        self.map.show(bg_iter)
    }

    pub fn clear(&mut self, vram: &mut VRamManager) {
        self.map.clear(vram);
    }
}

// Can remove once div_floor and div_ceil are stable
trait IntDivRoundingExt {
    fn div_floor_stable(self, other: Self) -> Self;
    fn div_ceil_stable(self, other: Self) -> Self;
}

impl IntDivRoundingExt for i32 {
    fn div_floor_stable(self, other: Self) -> Self {
        if self > 0 && other < 0 {
            (self - 1) / other - 1
        } else if self < 0 && other > 0 {
            (self + 1) / other - 1
        } else {
            self / other
        }
    }

    fn div_ceil_stable(self, other: Self) -> Self {
        if self > 0 && other > 0 {
            (self - 1) / other + 1
        } else if self < 0 && other < 0 {
            (self + 1) / other + 1
        } else {
            self / other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn div_floor_stable(_: &mut crate::Gba) {
        assert_eq!(12.div_floor_stable(5), 2);
        assert_eq!((-12).div_floor_stable(5), -3);
        assert_eq!(12.div_floor_stable(-5), -3);
        assert_eq!((-12).div_floor_stable(-5), 2);
    }

    #[test_case]
    fn div_ceil_stable(_: &mut crate::Gba) {
        assert_eq!(12.div_ceil_stable(5), 3);
        assert_eq!((-12).div_ceil_stable(5), -2);
        assert_eq!(12.div_ceil_stable(-5), -2);
        assert_eq!((-12).div_ceil_stable(-5), 3);
    }
}
