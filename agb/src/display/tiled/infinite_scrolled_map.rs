#![warn(missing_docs)]
use crate::{
    display::{GraphicsFrame, HEIGHT, Priority, WIDTH},
    fixnum::{Number, Rect, Vector2D, vec2},
};

use super::{RegularBackground, RegularBackgroundId, TileSet, TileSetting};

/// In tiles
const ONE_MORE_THAN_SCREEN_HEIGHT: i32 = HEIGHT / 8 + 1;
/// In tiles
const ONE_MORE_THAN_SCREEN_WIDTH: i32 = WIDTH / 8 + 1;

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

/// Create a 'virtual background' which is larger than the maximum size allowed by the Game Boy Advance.
///
/// The maximum background size in the Game Boy Advance is `64x64` tiles. Sometimes you want your game playing
/// space to be much larger than that. The `InfiniteScrolledMap` lets you pretend that the background is
/// as big as you need it to be, and it will lazily load the data into video RAM to ensure that the
/// screen is filled with the content that you expect it to. So the player won't know that tiles are being replaced
/// off-screen.
///
/// Note that the `InfiniteScrolledMap` only works with _regular backgrounds_ and not affine backgrounds.
///
/// You create an `InfiniteScrolledMap` by passing a [`RegularBackground`] you've created before. Then,
/// call [`set_scroll_pos()`](InfiniteScrolledMap::set_scroll_pos) to control the position of the scrolling.
///
/// See [here](https://agbrs.dev/examples/infinite_scrolled_map) for an example of how to use it.
pub struct InfiniteScrolledMap {
    map: RegularBackground,

    current_pos: Position,
}

impl InfiniteScrolledMap {
    /// Creates a new [`InfiniteScrolledMap`] taking ownership of the [`RegularBackground`]. Until you call
    /// [`set_scroll_pos()`](InfiniteScrolledMap::set_scroll_pos) calling [`.show()`](InfiniteScrolledMap::show) on this will
    /// do no more than calling `.show` would have on the `map`.
    ///
    /// You need to call [`set_scroll_pos()`](InfiniteScrolledMap::set_scroll_pos) in order to actually render something to the
    /// map.
    #[must_use]
    pub fn new(map: RegularBackground) -> Self {
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

    /// Scrolls the [`InfiniteScrolledMap`] to the provided location and does the minimum amount of
    /// rendering required in order to make the illusion that we have a map that's larger than the
    /// maximum background size provided by the Game Boy Advance. The behaviour is the same as
    /// [`RegularBackground::set_scroll_pos`] except without the wrapping behaviour.
    ///
    /// You should pass a function to the `tile` argument which, given a position, returns the tile
    /// that should be rendered in that location. Calling this with a new position that keeps some of
    /// the screen still visible will result in only the newly visible tiles being updated.
    ///
    /// The return value of this indicates whether the whole screen was updated, or if only part of
    /// the screen was updated. It can require quite a lot of CPU time to render the entire
    /// screen, so it is smeared across multiple frames to avoid dropping them if e.g. loading an
    /// entirely new set of tiles.
    ///
    /// * [`PartialUpdateStatus::Done`] is returned if the entire screen was updated.
    /// * [`PartialUpdateStatus::Continue`] is returned if only part of the screen was updated.
    ///
    /// It is recommended that you call this every frame, and then if update smearing has to happen,
    /// it won't happen for long and your players are unlikely to notice. You should also
    /// hide the scrolled map until the initial rendering is completed (by not calling
    /// [`InfiniteScrolledMap::show()`]) to hide the initial render.
    ///
    /// Do be aware that the provided `Vector2D<i32>` passed to the tile could be negative.
    pub fn set_scroll_pos(
        &mut self,
        new_pos: impl Into<Vector2D<i32>>,
        tile: impl Fn(Vector2D<i32>) -> (&'static TileSet<'static>, TileSetting),
    ) -> PartialUpdateStatus {
        let new_pos = new_pos.into();
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

    /// Returns whether the background has finished rendering.
    ///
    /// Will return the same value as whatever [`.set_scroll_pos()`](InfiniteScrolledMap::set_scroll_pos)
    /// returned last time.
    #[must_use]
    pub fn partial_update_status(&self) -> PartialUpdateStatus {
        match self.current_pos {
            Position::Current(_) => PartialUpdateStatus::Done,
            Position::Working { .. } | Position::None => PartialUpdateStatus::Continue,
        }
    }

    /// Gets the current scroll position.
    ///
    /// See [`RegularBackground::scroll_pos`] for more details
    #[must_use]
    pub fn scroll_pos(&self) -> Vector2D<i32> {
        self.map.scroll_pos()
    }

    /// Sets the priority of the underlying map.
    ///
    /// See [`RegularBackground::set_priority`] for more details
    pub fn set_priority(&mut self, priority: Priority) {
        self.map.set_priority(priority);
    }

    /// Gets the current priority of the underlying map.
    ///
    /// See [`RegularBackground::priority`] for more details.
    #[must_use]
    pub fn priority(&self) -> Priority {
        self.map.priority()
    }

    /// Shows this map on the given [`GraphicsFrame`].
    ///
    /// See [`RegularBackground::show`] for more details.
    pub fn show(&self, frame: &mut GraphicsFrame) -> RegularBackgroundId {
        self.map.show(frame)
    }

    /// Shows this map on the given [`GraphicsFrame`] if it has finished rendering.
    ///
    /// It takes multiple calls to [`.set_scroll_pos()`](InfiniteScrolledMap::set_scroll_pos) to fully
    /// update the screen. This method will only actually show the map if the full map has
    /// finished rendering.
    ///
    /// It'll return `None` if it didn't actually render the background, or `Some(backgroundId)` if
    /// it did with the same background id concept as in [`RegularBackground::show`].
    pub fn show_if_done(&self, frame: &mut GraphicsFrame) -> Option<RegularBackgroundId> {
        match self.partial_update_status() {
            PartialUpdateStatus::Done => Some(self.show(frame)),
            PartialUpdateStatus::Continue => None,
        }
    }
}

/// An indication as to whether scrolling is complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialUpdateStatus {
    /// The entire screen has been filled and is ready to show the player
    Done,
    /// There is still work to do to fully fill the screen. Maybe only a few rows of tiles have been rendered.
    Continue,
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

impl<T> IntDivRoundingExt<T> for Vector2D<T>
where
    T: IntDivRoundingExt<T> + Number,
{
    fn div_floor_stable(self, other: T) -> Self {
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
