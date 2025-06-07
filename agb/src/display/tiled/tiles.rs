use core::{cell::Cell, ptr::NonNull};

use alloc::{boxed::Box, rc::Rc, vec};

use crate::display::tiled::{Tile, TileFormat, TileIndex, VRAM_MANAGER};

#[derive(Clone)]
pub(crate) struct Tiles<T>
where
    T: TileInfo,
{
    tiles: Rc<TilesInner<T>>,
}

struct TilesInner<T>
where
    T: TileInfo,
{
    tile_data: Box<[T]>,
    /// This tracks where these tiles were last copied into. If it is None,
    /// then they either have never been copied, or they have been modified
    /// since they were last copied.
    ///
    /// This works as a cheap dirty flag.
    in_screenblock: Cell<Option<NonNull<u8>>>,
    colours: TileFormat,
}

impl<T> Clone for TilesInner<T>
where
    T: TileInfo,
{
    fn clone(&self) -> Self {
        for tile in &self.tile_data {
            if *tile != T::default() {
                VRAM_MANAGER.increase_reference(tile.tile_index(self.colours));
            }
        }

        Self {
            tile_data: self.tile_data.clone(),
            // We initialise this to None because the screenblock
            in_screenblock: Cell::new(None),
            colours: self.colours,
        }
    }
}

impl<T> Drop for TilesInner<T>
where
    T: TileInfo,
{
    fn drop(&mut self) {
        for tile in &self.tile_data {
            if *tile != T::default() {
                VRAM_MANAGER.remove_tile(tile.tile_index(self.colours));
            }
        }
    }
}

pub(crate) trait TileInfo: Default + Eq + Copy {
    fn tile_index(self, colours: TileFormat) -> TileIndex;
}

impl TileInfo for Tile {
    fn tile_index(self, colours: TileFormat) -> TileIndex {
        self.tile_index(colours)
    }
}

impl TileInfo for u8 {
    fn tile_index(self, _colours: TileFormat) -> TileIndex {
        TileIndex::EightBpp(self as u16)
    }
}

impl<T> Tiles<T>
where
    T: TileInfo,
{
    pub(crate) fn new(size: usize, colours: TileFormat) -> Self {
        let tiles = vec![T::default(); size].into_boxed_slice();

        Self {
            tiles: Rc::new(TilesInner {
                tile_data: tiles,
                in_screenblock: Cell::new(None),
                colours,
            }),
        }
    }

    pub(crate) fn set_tile(&mut self, pos: usize, tile: T) {
        let tile_data = Rc::make_mut(&mut self.tiles);
        tile_data.tile_data[pos] = tile;
        tile_data.in_screenblock.set(None);
    }

    pub(crate) fn as_ptr(&self) -> *const T {
        self.tiles().as_ptr()
    }

    pub(crate) fn colours(&self) -> TileFormat {
        self.tiles.colours
    }

    pub(crate) fn get(&self, index: usize) -> T {
        self.tiles()[index]
    }

    pub(crate) fn tiles(&self) -> &[T] {
        &self.tiles.tile_data
    }

    /// Returns whether or not this collection of tiles has been copied to the given
    /// screenblock pointer.
    pub(crate) fn is_dirty(&self, screenblock_ptr: NonNull<u8>) -> bool {
        self.tiles.in_screenblock.get() != Some(screenblock_ptr)
    }

    /// Assert that these tiles have been copied to the screenblock with the given pointer.
    /// The next call to is_dirty will return false if given the same screenblock pointer.
    pub(crate) fn clean(&self, screenblock_ptr: NonNull<u8>) {
        self.tiles.in_screenblock.set(Some(screenblock_ptr));
    }
}
