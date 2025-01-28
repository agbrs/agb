use agb_fixnum::{vec2, Rect, Vector2D};
use alloc::boxed::Box;

use crate::display::{self, HEIGHT, WIDTH};

use super::{BackgroundId, BackgroundIterator, RegularBackgroundTiles, TileSet, TileSetting};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialUpdateStatus {
    Done,
    Continue,
}

#[derive(Clone, Copy)]
enum Position {
    Current(Vector2D<i32>),
    Working {
        position: Vector2D<i32>,
        work_done: u32,
    },
    None,
}

impl Position {
    fn get(self) -> Option<Vector2D<i32>> {
        match self {
            Position::Current(pos) => Some(pos),
            Position::Working { position: pos, .. } => Some(pos * 8),
            Position::None => None,
        }
    }
}

pub struct InfiniteScrolledMap {
    map: RegularBackgroundTiles,

    current_pos: Position,
}

impl InfiniteScrolledMap {
    #[must_use]
    pub fn new(map: RegularBackgroundTiles) -> Self {
        Self {
            map,

            current_pos: Position::None,
        }
    }

    fn do_initial_case(
        &mut self,
        new_pos: Vector2D<i32>,
        tile: impl Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting),
    ) -> PartialUpdateStatus {
        let working = new_pos.div_floor_stable(8);

        let current_work_done = match self.current_pos {
            Position::Current(_) => unreachable!("Should never call do_initial_case with current"),
            Position::Working {
                position: original_working,
                work_done,
            } => {
                if original_working != working {
                    0
                } else {
                    work_done
                }
            }
            Position::None => 0,
        };

        const ROWS_TO_COPY_IN_ONE_CALL: u32 = 2;

        let top_left = working * 8;
        let offset = new_pos - top_left;

        self.map.set_scroll_pos(offset);

        for y in current_work_done..(current_work_done + ROWS_TO_COPY_IN_ONE_CALL) {
            for x in 0..(WIDTH / 8 + 1) {
                let pos = working + vec2(x, y as i32);
                let (tileset, tile_setting) = tile(pos);
                self.map.set_tile(vec2(x as u32, y), tileset, tile_setting);
            }
        }

        if current_work_done + ROWS_TO_COPY_IN_ONE_CALL < HEIGHT as u32 / 8 + 1 {
            self.current_pos = Position::Working {
                position: working,
                work_done: current_work_done + ROWS_TO_COPY_IN_ONE_CALL,
            };

            PartialUpdateStatus::Continue
        } else {
            self.current_pos = Position::Current(new_pos);
            PartialUpdateStatus::Done
        }
    }

    pub fn set_pos(
        &mut self,
        new_pos: Vector2D<i32>,
        tile: impl Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting),
    ) -> PartialUpdateStatus {
        // if the current pos is so far away from the new pos, we may as well start again
        if let Some(current_pos) = self.current_pos.get() {
            let distance = (current_pos - new_pos).abs();
            if distance.x >= WIDTH || distance.y >= HEIGHT {
                self.current_pos = Position::None;
            }
        }

        if matches!(self.current_pos, Position::Working { .. } | Position::None) {
            return self.do_initial_case(new_pos, tile);
        }

        PartialUpdateStatus::Done
    }

    pub fn commit(&mut self) {
        self.map.commit();
    }

    pub fn show(&self, bg_iter: &mut BackgroundIterator<'_>) -> BackgroundId {
        self.map.show(bg_iter)
    }
}

// Can remove once div_floor and div_ceil are stable
trait IntDivRoundingExt<Denominator> {
    fn div_floor_stable(self, other: Denominator) -> Self;
    fn div_ceil_stable(self, other: Denominator) -> Self;
}

impl IntDivRoundingExt<i32> for i32 {
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

impl IntDivRoundingExt<i32> for Vector2D<i32> {
    fn div_floor_stable(self, other: i32) -> Self {
        vec2(
            self.x.div_floor_stable(other),
            self.y.div_floor_stable(other),
        )
    }

    fn div_ceil_stable(self, other: i32) -> Self {
        vec2(self.x.div_ceil_stable(other), self.y.div_ceil_stable(other))
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
