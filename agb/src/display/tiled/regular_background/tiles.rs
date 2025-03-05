use alloc::{rc::Rc, vec};

use crate::display::tiled::{Tile, TileFormat, VRAM_MANAGER};

use super::RegularBackgroundSize;

pub(crate) struct Tiles {
    tiles: Rc<[Tile]>,
    colours: TileFormat,
}

impl Drop for Tiles {
    fn drop(&mut self) {
        if Rc::strong_count(&self.tiles) == 1 {
            for tile in self.tiles.iter() {
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
        Self {
            tiles: vec![Tile::default(); size.num_tiles()].into(),
            colours: format,
        }
    }

    pub(crate) fn tiles_mut(&mut self) -> &mut [Tile] {
        if Rc::strong_count(&self.tiles) > 1 {
            // the make_mut below is going to cause us to increase the reference count, so we should
            // mark every tile here as referenced again in the VRAM_MANAGER.
            for tile in self.tiles.iter() {
                if *tile != Tile::default() {
                    VRAM_MANAGER.increase_reference(tile.tile_index(self.colours));
                }
            }
        }

        Rc::make_mut(&mut self.tiles)
    }

    pub(crate) fn as_ptr(&self) -> *const Tile {
        self.tiles.as_ptr()
    }

    pub(crate) fn colours(&self) -> TileFormat {
        self.colours
    }

    pub(crate) fn get(&self, index: usize) -> Tile {
        self.tiles[index]
    }
}
