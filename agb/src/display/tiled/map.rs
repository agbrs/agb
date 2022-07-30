use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use crate::bitarray::Bitarray;
use crate::display::{Priority, DISPLAY_CONTROL};
use crate::dma::dma_copy16;
use crate::fixnum::Vector2D;
use crate::memory_mapped::MemoryMapped;

use super::{RegularBackgroundSize, Tile, TileSet, TileSetting, VRamManager};

use alloc::{vec, vec::Vec};

pub struct RegularMap {
    background_id: u8,

    screenblock: u8,
    x_scroll: u16,
    y_scroll: u16,
    priority: Priority,

    tiles: Vec<Tile>,
    tiles_dirty: bool,

    size: RegularBackgroundSize,
}

pub const TRANSPARENT_TILE_INDEX: u16 = (1 << 10) - 1;

impl RegularMap {
    pub(crate) fn new(
        background_id: u8,
        screenblock: u8,
        priority: Priority,
        size: RegularBackgroundSize,
    ) -> Self {
        Self {
            background_id,

            screenblock,
            x_scroll: 0,
            y_scroll: 0,
            priority,

            tiles: vec![Default::default(); size.num_tiles()],
            tiles_dirty: true,

            size,
        }
    }

    pub fn set_tile(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<u16>,
        tileset: &TileSet<'_>,
        tile_setting: TileSetting,
    ) {
        let pos = self.size.gba_offset(pos);

        let old_tile = self.tiles[pos];
        if old_tile != Tile::default() {
            vram.remove_tile(old_tile.tile_index());
        }

        let tile_index = tile_setting.index();

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = vram.add_tile(tileset, tile_index);
            Tile::new(new_tile_idx, tile_setting)
        } else {
            Tile::default()
        };

        if old_tile == new_tile {
            // no need to mark as dirty if nothing changes
            return;
        }

        self.tiles[pos] = new_tile;
        self.tiles_dirty = true;
    }

    pub fn clear(&mut self, vram: &mut VRamManager) {
        for tile in self.tiles.iter_mut() {
            if *tile != Tile::default() {
                vram.remove_tile(tile.tile_index());
            }

            *tile = Tile::default();
        }
    }

    pub fn show(&mut self) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode | (1 << (self.background_id + 0x08));
        DISPLAY_CONTROL.set(new_mode);
    }

    pub fn hide(&mut self) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode & !(1 << (self.background_id + 0x08));
        DISPLAY_CONTROL.set(new_mode);
    }

    pub fn commit(&mut self, vram: &mut VRamManager) {
        let new_bg_control_value = (self.priority as u16)
            | (u16::from(self.screenblock) << 8)
            | (self.size.size_flag() << 14);

        self.bg_control_register().set(new_bg_control_value);
        self.bg_h_offset().set(self.x_scroll);
        self.bg_v_offset().set(self.y_scroll);

        let screenblock_memory = self.screenblock_memory();

        if self.tiles_dirty {
            unsafe {
                dma_copy16(
                    self.tiles.as_ptr() as *const u16,
                    screenblock_memory,
                    self.size.num_tiles(),
                );
            }
        }

        vram.gc();

        self.tiles_dirty = false;
    }

    pub fn set_scroll_pos(&mut self, pos: Vector2D<u16>) {
        self.x_scroll = pos.x;
        self.y_scroll = pos.y;
    }

    #[must_use]
    pub fn scroll_pos(&self) -> Vector2D<u16> {
        (self.x_scroll, self.y_scroll).into()
    }

    pub(crate) fn size(&self) -> RegularBackgroundSize {
        self.size
    }

    const fn bg_control_register(&self) -> MemoryMapped<u16> {
        unsafe { MemoryMapped::new(0x0400_0008 + 2 * self.background_id as usize) }
    }

    const fn bg_h_offset(&self) -> MemoryMapped<u16> {
        unsafe { MemoryMapped::new(0x0400_0010 + 4 * self.background_id as usize) }
    }

    const fn bg_v_offset(&self) -> MemoryMapped<u16> {
        unsafe { MemoryMapped::new(0x0400_0012 + 4 * self.background_id as usize) }
    }

    const fn screenblock_memory(&self) -> *mut u16 {
        (0x0600_0000 + 0x1000 * self.screenblock as usize / 2) as *mut u16
    }
}

pub struct MapLoan<'a, T> {
    map: T,
    background_id: u8,
    screenblock_id: u8,
    screenblock_length: u8,
    regular_map_list: &'a RefCell<Bitarray<1>>,
    screenblock_list: &'a RefCell<Bitarray<1>>,
}

impl<'a, T> Deref for MapLoan<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'a, T> DerefMut for MapLoan<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<'a, T> MapLoan<'a, T> {
    pub(crate) fn new(
        map: T,
        background_id: u8,
        screenblock_id: u8,
        screenblock_length: u8,
        regular_map_list: &'a RefCell<Bitarray<1>>,
        screenblock_list: &'a RefCell<Bitarray<1>>,
    ) -> Self {
        MapLoan {
            map,
            background_id,
            screenblock_id,
            screenblock_length,
            regular_map_list,
            screenblock_list,
        }
    }
}

impl<'a, T> Drop for MapLoan<'a, T> {
    fn drop(&mut self) {
        self.regular_map_list
            .borrow_mut()
            .set(self.background_id as usize, false);

        let mut screenblock_list = self.screenblock_list.borrow_mut();

        for i in self.screenblock_id..self.screenblock_id + self.screenblock_length {
            screenblock_list.set(i as usize, false);
        }
    }
}
