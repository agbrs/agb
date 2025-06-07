use core::{cell::RefCell, ptr::NonNull};

use alloc::{boxed::Box, rc::Rc, vec};

use crate::display::tiled::{Tile, TileFormat, VRAM_MANAGER};

use super::RegularBackgroundSize;

pub(crate) struct Tiles {
    tiles: Rc<TilesInner>,
    colours: TileFormat,
}

struct TilesInner {
    tile_data: Box<[Tile]>,
    in_screenblock: RefCell<Option<NonNull<u8>>>,
}

impl Clone for TilesInner {
    fn clone(&self) -> Self {
        Self {
            tile_data: self.tile_data.clone(),
            in_screenblock: RefCell::new(None),
        }
    }
}

impl Drop for Tiles {
    fn drop(&mut self) {
        if Rc::strong_count(&self.tiles) == 1 {
            for tile in self.tiles().iter() {
                if *tile != Tile::default() {
                    VRAM_MANAGER.remove_tile(tile.tile_index(self.colours));
                }
            }
        }
    }
}

impl Clone for Tiles {
    fn clone(&self) -> Self {
        Self {
            tiles: Rc::clone(&self.tiles),
            colours: self.colours,
        }
    }
}

impl Tiles {
    pub(crate) fn new(size: RegularBackgroundSize, format: TileFormat) -> Self {
        let tiles = vec![Tile::default(); size.num_tiles()].into();
        Self {
            tiles: Rc::new(TilesInner {
                tile_data: tiles,
                in_screenblock: RefCell::new(None),
            }),
            colours: format,
        }
    }

    pub(crate) fn set_tile(&mut self, pos: usize, tile: Tile) {
        if Rc::strong_count(&self.tiles) > 1 {
            // the make_mut below is going to cause us to increase the reference count, so we should
            // mark every tile here as referenced again in the VRAM_MANAGER.
            for tile in self.tiles().iter() {
                if *tile != Tile::default() {
                    VRAM_MANAGER.increase_reference(tile.tile_index(self.colours));
                }
            }
        }

        let tile_data = Rc::make_mut(&mut self.tiles);
        tile_data.tile_data[pos] = tile;
        tile_data.in_screenblock.replace(None);
    }

    pub(crate) fn as_ptr(&self) -> *const Tile {
        self.tiles().as_ptr()
    }

    pub(crate) fn colours(&self) -> TileFormat {
        self.colours
    }

    pub(crate) fn get(&self, index: usize) -> Tile {
        self.tiles()[index]
    }

    pub(crate) fn tiles(&self) -> &[Tile] {
        &self.tiles.tile_data
    }

    pub(crate) fn is_dirty(&self, screenblock_ptr: NonNull<u8>) -> bool {
        *self.tiles.in_screenblock.borrow() != Some(screenblock_ptr)
    }

    pub(crate) fn clean(&self, screenblock_ptr: NonNull<u8>) {
        self.tiles.in_screenblock.replace(Some(screenblock_ptr));
    }
}
