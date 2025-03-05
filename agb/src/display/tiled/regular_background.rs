use core::{
    alloc::{Allocator, Layout},
    mem,
    ptr::NonNull,
};

use agb_fixnum::Vector2D;
use alloc::{rc::Rc, vec};

use crate::display::{GraphicsFrame, Priority, tile_data::TileData};

use super::{
    BackgroundControlRegister, BackgroundId, RegularBackgroundCommitData, RegularBackgroundData,
    SCREENBLOCK_SIZE, ScreenblockAllocator, TRANSPARENT_TILE_INDEX, Tile, TileFormat, TileSet,
    TileSetting, VRAM_MANAGER, VRAM_START,
};

use bilge::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum RegularBackgroundSize {
    Background32x32 = 0,
    Background64x32 = 1,
    Background32x64 = 2,
    Background64x64 = 3,
}

impl RegularBackgroundSize {
    const fn width(self) -> usize {
        match self {
            RegularBackgroundSize::Background32x32 => 32,
            RegularBackgroundSize::Background64x32 => 64,
            RegularBackgroundSize::Background32x64 => 32,
            RegularBackgroundSize::Background64x64 => 64,
        }
    }

    const fn height(self) -> usize {
        match self {
            RegularBackgroundSize::Background32x32 => 32,
            RegularBackgroundSize::Background64x32 => 32,
            RegularBackgroundSize::Background32x64 => 64,
            RegularBackgroundSize::Background64x64 => 64,
        }
    }

    const fn size_in_bytes(self) -> usize {
        self.num_tiles() * mem::size_of::<Tile>()
    }

    fn layout(self) -> Layout {
        Layout::from_size_align(self.size_in_bytes(), SCREENBLOCK_SIZE)
            .expect("failed to create layout, should never happen")
    }

    const fn num_tiles(self) -> usize {
        self.width() * self.height()
    }

    const fn gba_offset(self, pos: Vector2D<i32>) -> usize {
        let x_mod = (pos.x & (self.width() as i32 - 1)) as u32;
        let y_mod = (pos.y & (self.height() as i32 - 1)) as u32;

        let screenblock = (x_mod / 32) + (y_mod / 32) * (self.width() as u32 / 32);

        let pos = screenblock * 32 * 32 + (x_mod % 32 + 32 * (y_mod % 32));

        pos as usize
    }
}

pub(crate) struct RegularBackgroundScreenblock {
    ptr: NonNull<u8>,
    size: RegularBackgroundSize,
}

impl RegularBackgroundScreenblock {
    pub(crate) unsafe fn copy_tiles(&self, tiles: &Tiles) {
        unsafe {
            self.ptr
                .as_ptr()
                .cast::<Tile>()
                .copy_from_nonoverlapping(tiles.tiles.as_ptr(), self.size.num_tiles());
        }
    }
}

pub struct RegularBackgroundTiles {
    priority: Priority,

    tiles: Tiles,
    screenblock: Rc<RegularBackgroundScreenblock>,

    is_dirty: bool,

    scroll: Vector2D<i32>,
}

impl RegularBackgroundTiles {
    #[must_use]
    pub fn new(priority: Priority, size: RegularBackgroundSize, colours: TileFormat) -> Self {
        let screenblock_ptr = ScreenblockAllocator
            .allocate(size.layout())
            .expect("Not enough space to allocate for background")
            .cast();

        Self {
            priority,

            tiles: Tiles {
                tiles: vec![Tile::default(); size.num_tiles()].into(),
                colours,
            },
            is_dirty: true,

            scroll: Vector2D::default(),

            screenblock: Rc::new(RegularBackgroundScreenblock {
                ptr: screenblock_ptr,
                size,
            }),
        }
    }

    pub fn set_scroll_pos(&mut self, scroll: impl Into<Vector2D<i32>>) {
        self.scroll = scroll.into();
    }

    #[must_use]
    pub fn scroll_pos(&self) -> Vector2D<i32> {
        self.scroll
    }

    pub fn set_tile(
        &mut self,
        pos: impl Into<Vector2D<i32>>,
        tileset: &TileSet<'_>,
        tile_setting: TileSetting,
    ) {
        assert_eq!(
            tileset.format(),
            self.tiles.colours,
            "Cannot set a {:?} colour tile on a {:?} colour background",
            tileset.format(),
            self.tiles.colours
        );

        let pos = self.screenblock.size.gba_offset(pos.into());
        self.set_tile_at_pos(pos, tileset, tile_setting);
    }

    pub fn fill_with(&mut self, tile_data: &TileData) {
        assert!(
            tile_data.tile_settings.len() >= 20 * 30,
            "Don't have a full screen's worth of tile data"
        );

        assert_eq!(
            tile_data.tiles.format(),
            self.tiles.colours,
            "Cannot set a {:?} colour tile on a {:?} colour background",
            tile_data.tiles.format(),
            self.tiles.colours
        );

        for y in 0..20 {
            for x in 0..30 {
                let tile_id = y * 30 + x;
                let tile_pos = y * 32 + x;
                self.set_tile_at_pos(tile_pos, &tile_data.tiles, tile_data.tile_settings[tile_id]);
            }
        }
    }

    fn set_tile_at_pos(&mut self, pos: usize, tileset: &TileSet<'_>, tile_setting: TileSetting) {
        let old_tile = self.tiles.tiles[pos];
        if old_tile != Tile::default() {
            VRAM_MANAGER.remove_tile(old_tile.tile_index(self.tiles.colours));
        }

        let tile_index = tile_setting.index();

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = VRAM_MANAGER.add_tile(tileset, tile_index);
            Tile::new(new_tile_idx, tile_setting)
        } else {
            Tile::default()
        };

        if old_tile == new_tile {
            // no need to mark as dirty if nothing changes
            return;
        }

        self.tiles.tiles_mut()[pos] = new_tile;
        self.is_dirty = true;
    }

    pub fn show(&self, frame: &mut GraphicsFrame<'_>) -> BackgroundId {
        let commit_data = if self.is_dirty {
            Some(RegularBackgroundCommitData {
                tiles: self.tiles.clone(),
                screenblock: Rc::clone(&self.screenblock),
            })
        } else {
            None
        };

        frame.bg_frame.set_next_regular(RegularBackgroundData {
            bg_ctrl: self.bg_ctrl_value(),
            scroll_offset: Vector2D::new(self.scroll.x as u16, self.scroll.y as u16),
            commit_data,
        })
    }

    #[must_use]
    pub fn size(&self) -> RegularBackgroundSize {
        self.screenblock.size
    }

    fn bg_ctrl_value(&self) -> BackgroundControlRegister {
        let mut background_control_register = BackgroundControlRegister::default();

        background_control_register.set_tile_format(self.tiles.colours.into());
        background_control_register.set_priority(self.priority.into());
        background_control_register.set_screen_base_block(u5::new(self.screen_base_block() as u8));
        background_control_register.set_screen_size(self.size().into());

        background_control_register
    }

    fn screen_base_block(&self) -> u16 {
        let screenblock_location = self.screenblock.ptr.as_ptr() as usize;
        ((screenblock_location - VRAM_START) / SCREENBLOCK_SIZE) as u16
    }
}

impl Drop for RegularBackgroundScreenblock {
    fn drop(&mut self) {
        unsafe { ScreenblockAllocator.deallocate(self.ptr.cast(), self.size.layout()) };
    }
}

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
    fn tiles_mut(&mut self) -> &mut [Tile] {
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
}

#[cfg(test)]
mod test;
