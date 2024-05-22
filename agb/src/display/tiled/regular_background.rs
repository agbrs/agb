use core::{
    alloc::{Allocator, Layout},
    mem,
    ptr::NonNull,
};

use agb_fixnum::Vector2D;
use alloc::{vec, vec::Vec};

use crate::display::Priority;

use super::{
    ScreenblockAllocator, Tile, TileFormat, TileSet, TileSetting, VRamManager, SCREENBLOCK_SIZE,
    TRANSPARENT_TILE_INDEX,
};

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
            RegularBackgroundSize::Background64x32 => 64,
            RegularBackgroundSize::Background32x64 => 64,
            RegularBackgroundSize::Background64x64 => 32,
        }
    }

    const fn size_in_bytes(self) -> usize {
        self.num_tiles() * mem::size_of::<Tile>()
    }

    const fn layout(self) -> Layout {
        match Layout::from_size_align(self.size_in_bytes(), SCREENBLOCK_SIZE) {
            Ok(layout) => layout,
            Err(_) => panic!("failed to create layout, should never happen"),
        }
    }

    const fn num_tiles(self) -> usize {
        self.width() * self.height()
    }

    const fn gba_offset(self, pos: Vector2D<u16>) -> usize {
        let x_mod = pos.x & (self.width() as u16 - 1);
        let y_mod = pos.y & (self.height() as u16 - 1);

        let screenblock = (x_mod / 32) + (y_mod / 32) * (self.width() as u16 / 32);

        let pos = screenblock * 32 * 32 + (x_mod % 32 + 32 * (y_mod % 32));

        pos as usize
    }
}

pub struct RegularBackgroundTiles {
    priority: Priority,
    size: RegularBackgroundSize,
    colours: TileFormat,

    tiles: Vec<Tile>,
    is_dirty: bool,

    screenblock_ptr: NonNull<[Tile]>,
}

impl RegularBackgroundTiles {
    pub fn new(priority: Priority, size: RegularBackgroundSize, colours: TileFormat) -> Self {
        let screenblock_ptr = ScreenblockAllocator
            .allocate(size.layout())
            .expect("Not enough space to allocate for background");

        Self {
            priority,
            size,
            colours,

            tiles: vec![Tile::default(); size.num_tiles()],
            is_dirty: true,

            screenblock_ptr: unsafe { mem::transmute(screenblock_ptr) },
        }
    }

    pub fn set_tile(
        &mut self,
        vram: &mut VRamManager,
        pos: impl Into<Vector2D<u16>>,
        tileset: &TileSet<'_>,
        tile_setting: TileSetting,
    ) {
        assert_eq!(
            tileset.format(),
            self.colours,
            "Cannot set a {:?} colour tile on a {:?} colour background",
            tileset.format(),
            self.colours
        );

        let pos = self.size.gba_offset(pos.into());
        self.set_tile_at_pos(vram, pos, tileset, tile_setting);
    }

    fn set_tile_at_pos(
        &mut self,
        vram: &mut VRamManager,
        pos: usize,
        tileset: &TileSet<'_>,
        tile_setting: TileSetting,
    ) {
        let old_tile = self.tiles[pos];
        if old_tile != Tile::default() {
            vram.remove_tile(old_tile.tile_index(self.colours));
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
        self.is_dirty = true;
    }
}

impl Drop for RegularBackgroundTiles {
    fn drop(&mut self) {
        unsafe { ScreenblockAllocator.deallocate(self.screenblock_ptr.cast(), self.size.layout()) };

        // TODO: Deallocate the tiles
    }
}
