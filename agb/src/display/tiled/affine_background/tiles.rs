use core::{cell::Cell, ptr::NonNull};

use alloc::{boxed::Box, rc::Rc, vec};

use crate::display::tiled::{TileIndex, VRAM_MANAGER};

use super::AffineBackgroundSize;

pub(crate) struct Tiles {
    tiles: Rc<TilesInner>,
}

struct TilesInner {
    tiles: Box<[u8]>,
    in_screenblock: Cell<Option<NonNull<u8>>>,
}

impl Clone for TilesInner {
    fn clone(&self) -> Self {
        Self {
            tiles: self.tiles.clone(),
            in_screenblock: Cell::new(None),
        }
    }
}

impl Drop for Tiles {
    fn drop(&mut self) {
        if Rc::strong_count(&self.tiles) == 1 {
            for tile in self.tiles().iter() {
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
        let tiles = vec![0; size.num_tiles()].into_boxed_slice();
        Self {
            tiles: Rc::new(TilesInner {
                tiles,
                in_screenblock: Cell::new(None),
            }),
        }
    }

    pub(crate) fn set_tile(&mut self, pos: usize, idx: u8) {
        if Rc::strong_count(&self.tiles) > 1 {
            // the make_mut below is going to cause us to increase the reference count, so we should
            // mark every tile here as referenced again in the VRAM_MANAGER.
            for tile in self.tiles().iter() {
                if *tile != 0 {
                    VRAM_MANAGER.increase_reference(TileIndex::EightBpp(*tile as u16));
                }
            }
        }

        let tile_data = Rc::make_mut(&mut self.tiles);
        tile_data.tiles[pos] = idx;
        tile_data.in_screenblock.set(None);
    }

    pub(crate) fn tiles(&self) -> &[u8] {
        &self.tiles.tiles
    }

    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.tiles().as_ptr()
    }

    pub(crate) fn get(&self, index: usize) -> u8 {
        self.tiles()[index]
    }

    pub(crate) fn is_dirty(&self, screenblock_ptr: NonNull<u8>) -> bool {
        self.tiles.in_screenblock.get() != Some(screenblock_ptr)
    }

    pub(crate) fn clean(&self, screenblock_ptr: NonNull<u8>) {
        self.tiles.in_screenblock.set(Some(screenblock_ptr));
    }
}
