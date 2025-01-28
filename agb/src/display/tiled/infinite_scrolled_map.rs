use agb_fixnum::{vec2, Rect, Vector2D};

use crate::display::{HEIGHT, WIDTH};

use super::{BackgroundId, BackgroundIterator, RegularBackgroundTiles, TileSet, TileSetting};

/// In tiles
const ONE_MORE_THAN_SCREEN_HEIGHT: i32 = HEIGHT / 8 + 1;
/// In tiles
const ONE_MORE_THAN_SCREEN_WIDTH: i32 = WIDTH / 8 + 1;

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

        for y in current_work_done..(current_work_done + ROWS_TO_COPY_IN_ONE_CALL) {
            for x in 0..(WIDTH / 8 + 1) {
                let pos = working + vec2(x, y as i32);
                let (tileset, tile_setting) = tile(pos);
                self.map.set_tile(pos, tileset, tile_setting);
            }
        }

        if current_work_done + ROWS_TO_COPY_IN_ONE_CALL < ONE_MORE_THAN_SCREEN_HEIGHT as u32 {
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

    fn update_rectangle(
        &mut self,
        rectangle: Rect<i32>,
        tile: impl Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting),
    ) {
        for pos in rectangle.iter() {
            let (tileset, tile_setting) = tile(pos);

            self.map.set_tile(pos, tileset, tile_setting);
        }
    }

    fn incremental_update(
        &mut self,
        old_pos: Vector2D<i32>,
        new_pos: Vector2D<i32>,
        tile: impl Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting),
    ) -> PartialUpdateStatus {
        let old_working = old_pos.div_floor_stable(8);
        let new_working = new_pos.div_floor_stable(8);

        if old_working == new_working {
            return PartialUpdateStatus::Done;
        }

        if old_working.x > new_working.x {
            self.update_rectangle(
                Rect::new(
                    new_working,
                    vec2(old_working.x - new_working.x, ONE_MORE_THAN_SCREEN_HEIGHT),
                ),
                &tile,
            );
        }

        if old_working.x < new_working.x {
            self.update_rectangle(
                Rect::new(
                    old_working + vec2(ONE_MORE_THAN_SCREEN_WIDTH, 0),
                    vec2(new_working.x - old_working.x, ONE_MORE_THAN_SCREEN_HEIGHT),
                ),
                &tile,
            );
        }

        if old_working.y > new_working.y {
            self.update_rectangle(
                Rect::new(
                    new_working,
                    vec2(ONE_MORE_THAN_SCREEN_WIDTH, old_working.y - new_working.y),
                ),
                &tile,
            );
        }

        if old_working.y < new_working.y {
            self.update_rectangle(
                Rect::new(
                    old_working + vec2(0, ONE_MORE_THAN_SCREEN_HEIGHT),
                    vec2(ONE_MORE_THAN_SCREEN_WIDTH, new_working.y - old_working.y),
                ),
                &tile,
            );
        }

        self.current_pos = Position::Current(new_pos);

        PartialUpdateStatus::Done
    }

    pub fn set_pos(
        &mut self,
        new_pos: Vector2D<i32>,
        tile: impl Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting),
    ) -> PartialUpdateStatus {
        self.map.set_scroll_pos(new_pos);

        // if the current pos is so far away from the new pos, we may as well start again
        if let Some(current_pos) = self.current_pos.get() {
            let distance = (current_pos - new_pos).abs();
            if distance.x >= WIDTH || distance.y >= HEIGHT {
                self.current_pos = Position::None;
            }
        }

        match self.current_pos {
            Position::Current(old_pos) => self.incremental_update(old_pos, new_pos, tile),
            Position::Working { .. } | Position::None => self.do_initial_case(new_pos, tile),
        }
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
}

impl IntDivRoundingExt<i32> for Vector2D<i32> {
    fn div_floor_stable(self, other: i32) -> Self {
        vec2(
            self.x.div_floor_stable(other),
            self.y.div_floor_stable(other),
        )
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
}
