use core::{cell::Cell, ptr::NonNull};

use alloc::{boxed::Box, rc::Rc, vec};

use crate::display::tiled::{Tile, TileFormat, VRAM_MANAGER};

use super::RegularBackgroundSize;

#[derive(Clone)]
pub(crate) struct Tiles {
    tiles: Rc<TilesInner>,
}

struct TilesInner {
    tile_data: Box<[Tile]>,
    /// This tracks where these tiles were last copied into. If it is None,
    /// then they either have never been copied, or they have been modified
    /// since they were last copied.
    ///
    /// This works as a cheap dirty flag.
    in_screenblock: Cell<Option<NonNull<u8>>>,
    colours: TileFormat,
}

impl Clone for TilesInner {
    fn clone(&self) -> Self {
        for tile in &self.tile_data {
            if *tile != Tile::default() {
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

impl Drop for TilesInner {
    fn drop(&mut self) {
        for tile in &self.tile_data {
            if *tile != Tile::default() {
                VRAM_MANAGER.remove_tile(tile.tile_index(self.colours));
            }
        }
    }
}

impl Tiles {
    pub(crate) fn new(size: RegularBackgroundSize, format: TileFormat) -> Self {
        let tiles = vec![Tile::default(); size.num_tiles()].into();
        Self {
            tiles: Rc::new(TilesInner {
                tile_data: tiles,
                in_screenblock: Cell::new(None),
                colours: format,
            }),
        }
    }

    /// Sets the tile at the given position. Will also mark this set of tiles as dirty
    pub(crate) fn set_tile(&mut self, pos: usize, tile: Tile) {
        let tile_data = Rc::make_mut(&mut self.tiles);
        tile_data.tile_data[pos] = tile;
        tile_data.in_screenblock.set(None);
    }

    pub(crate) fn as_ptr(&self) -> *const Tile {
        self.tiles().as_ptr()
    }

    pub(crate) fn colours(&self) -> TileFormat {
        self.tiles.colours
    }

    pub(crate) fn get(&self, index: usize) -> Tile {
        self.tiles()[index]
    }

    pub(crate) fn tiles(&self) -> &[Tile] {
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
