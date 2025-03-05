use alloc::{rc::Rc, vec};

use crate::display::tiled::{TileIndex, VRAM_MANAGER};

use super::AffineBackgroundSize;

pub(crate) struct Tiles {
    tiles: Rc<[u8]>,
}

impl Drop for Tiles {
    fn drop(&mut self) {
        if Rc::strong_count(&self.tiles) == 1 {
            for tile in self.tiles.iter() {
                if *tile != 0 {
                    VRAM_MANAGER.remove_tile(TileIndex::EightBpp(*tile as u16));
                }
            }
        }
    }
}

impl Clone for Tiles {
    fn clone(&self) -> Self {
        Self {
            tiles: Rc::clone(&self.tiles),
        }
    }
}

impl Tiles {
    pub(crate) fn new(size: AffineBackgroundSize) -> Self {
        Self {
            tiles: vec![0; size.num_tiles()].into(),
        }
    }

    pub(crate) fn tiles_mut(&mut self) -> &mut [u8] {
        if Rc::strong_count(&self.tiles) > 1 {
            // the make_mut below is going to cause us to increase the reference count, so we should
            // mark every tile here as referenced again in the VRAM_MANAGER.
            for tile in self.tiles.iter() {
                if *tile != 0 {
                    VRAM_MANAGER.increase_reference(TileIndex::EightBpp(*tile as u16));
                }
            }
        }

        Rc::make_mut(&mut self.tiles)
    }

    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.tiles.as_ptr()
    }

    pub(crate) fn get(&self, index: usize) -> u8 {
        self.tiles[index]
    }
}
