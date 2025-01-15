use agb_fixnum::Vector2D;
use alloc::boxed::Box;

use super::{RegularBackgroundTiles, TileSet, TileSetting, VRamManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialUpdateStatus {
    Done,
    Continue,
}

pub struct InfiniteScrolledMap<'a> {
    map: RegularBackgroundTiles,
    tile: Box<dyn Fn(Vector2D<i32>) -> (&'a TileSet<'a>, TileSetting) + 'a>,

    current_pos: Vector2D<i32>,
    offset: Vector2D<i32>,

    copied_up_to: i32,
}

impl<'a> InfiniteScrolledMap<'a> {
    pub fn new(
        map: RegularBackgroundTiles,
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

    pub fn init_partial(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<i32>,
    ) -> PartialUpdateStatus {
        self.current_pos = pos;

        let x_start = self.current_pos.x.div_floor_stable(8);

        PartialUpdateStatus::Continue
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
