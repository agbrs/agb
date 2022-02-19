use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use crate::bitarray::Bitarray;
use crate::display::{self, Priority, DISPLAY_CONTROL};
use crate::fixnum::Vector2D;
use crate::memory_mapped::{MemoryMapped, MemoryMapped1DArray};

use super::{Tile, TileSetReference, TileSetting, VRamManager};

pub struct RegularMap {
    background_id: u8,

    screenblock: u8,
    x_scroll: u16,
    y_scroll: u16,
    priority: Priority,

    tiles: [Tile; 32 * 32],
    tiles_dirty: bool,
}

pub const TRANSPARENT_TILE_INDEX: u16 = (1 << 10) - 1;

impl RegularMap {
    pub(crate) fn new(background_id: u8, screenblock: u8, priority: Priority) -> Self {
        Self {
            background_id,

            screenblock,
            x_scroll: 0,
            y_scroll: 0,
            priority,

            tiles: [Tile::default(); 32 * 32],
            tiles_dirty: true,
        }
    }

    pub fn set_tile(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<u16>,
        tileset_ref: TileSetReference,
        tile_setting: TileSetting,
    ) {
        let pos = (pos.x + pos.y * 32) as usize;

        let old_tile = self.tiles[pos];
        if old_tile != Tile::default() {
            vram.remove_tile(old_tile.tile_index());
        }

        let tile_index = tile_setting.index();

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = vram.add_tile(tileset_ref, tile_index);
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
            vram.remove_tile(tile.tile_index());

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

    pub fn commit(&mut self) {
        let new_bg_control_value = (self.priority as u16) | ((self.screenblock as u16) << 8);

        self.bg_control_register().set(new_bg_control_value);
        self.bg_h_offset().set(self.x_scroll);
        self.bg_v_offset().set(self.y_scroll);

        if !self.tiles_dirty {
            return;
        }

        let screenblock_memory = self.screenblock_memory();

        let scroll_pos = self.get_scroll_pos();
        let start_x = scroll_pos.x / 8;
        let end_x = div_ceil(scroll_pos.x as i32 + display::WIDTH, 8) as u16 + 1;

        let start_y = scroll_pos.y / 8;
        let end_y = div_ceil(scroll_pos.y as i32 + display::HEIGHT, 8) as u16 + 1;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let id = y.rem_euclid(32) * 32 + x.rem_euclid(32);
                screenblock_memory.set(id as usize, self.tiles[id as usize].0);
            }
        }

        self.tiles_dirty = false;
    }

    pub fn set_scroll_pos(&mut self, pos: Vector2D<u16>) {
        self.x_scroll = pos.x;
        self.y_scroll = pos.y;
    }

    pub fn get_scroll_pos(&self) -> Vector2D<u16> {
        (self.x_scroll, self.y_scroll).into()
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

    const fn screenblock_memory(&self) -> MemoryMapped1DArray<u16, { 32 * 32 }> {
        unsafe { MemoryMapped1DArray::new(0x0600_0000 + 0x1000 * self.screenblock as usize / 2) }
    }
}

pub struct MapLoan<'a, T> {
    map: T,
    background_id: u8,
    regular_map_list: &'a RefCell<Bitarray<1>>,
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
        regular_map_list: &'a RefCell<Bitarray<1>>,
    ) -> Self {
        MapLoan {
            map,
            background_id,
            regular_map_list,
        }
    }
}

impl<'a, T> Drop for MapLoan<'a, T> {
    fn drop(&mut self) {
        self.regular_map_list
            .borrow_mut()
            .set(self.background_id as usize, false);
    }
}

fn div_ceil(x: i32, y: i32) -> i32 {
    if x > 0 && y > 0 {
        (x - 1) / y + 1
    } else if x < 0 && y < 0 {
        (x + 1) / y + 1
    } else {
        x / y
    }
}
