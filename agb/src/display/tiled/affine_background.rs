use alloc::{alloc::Allocator, vec, vec::Vec};
use bilge::prelude::*;
use core::{alloc::Layout, ptr::NonNull};

use crate::{
    display::{GraphicsFrame, Priority, affine::AffineMatrixBackground, tiled::TileFormat},
    fixnum::{Num, Vector2D},
};

use super::{
    AffineBackgroundData, AffineBackgroundId, BackgroundControlRegister, SCREENBLOCK_SIZE,
    ScreenblockAllocator, TRANSPARENT_TILE_INDEX, TileIndex, TileSet, VRAM_MANAGER, VRAM_START,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum AffineBackgroundSize {
    Background16x16 = 0,
    Background32x32 = 1,
    Background64x64 = 2,
    Background128x128 = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum AffineBackgroundWrapBehaviour {
    NoWrap = 0,
    Wrap = 1,
}

impl AffineBackgroundSize {
    fn width(self) -> usize {
        match self {
            AffineBackgroundSize::Background16x16 => 16,
            AffineBackgroundSize::Background32x32 => 32,
            AffineBackgroundSize::Background64x64 => 64,
            AffineBackgroundSize::Background128x128 => 128,
        }
    }

    fn num_tiles(self) -> usize {
        self.width() * self.width()
    }

    fn layout(self) -> Layout {
        Layout::from_size_align(self.num_tiles(), SCREENBLOCK_SIZE)
            .expect("Failed to create layout, should never happen")
    }

    fn gba_offset(self, pos: Vector2D<i32>) -> usize {
        let x_mod = pos.x & (self.width() as i32 - 1);
        let y_mod = pos.y & (self.width() as i32 - 1);

        let pos = x_mod + (self.width() as i32 * y_mod);

        pos as usize
    }
}

pub struct AffineBackgroundTiles {
    priority: Priority,
    size: AffineBackgroundSize,

    // Affine backgrounds only store the tile id
    tiles: Vec<u8>,
    is_dirty: bool,

    scroll: Vector2D<Num<i32, 8>>,
    screenblock_ptr: NonNull<u8>,

    transform: AffineMatrixBackground,
    wrap_behaviour: AffineBackgroundWrapBehaviour,
}

impl AffineBackgroundTiles {
    #[must_use]
    pub fn new(
        priority: Priority,
        size: AffineBackgroundSize,
        wrap_behaviour: AffineBackgroundWrapBehaviour,
    ) -> Self {
        let screenblock_ptr = ScreenblockAllocator
            .allocate(size.layout())
            .expect("Not enough space to allocate affine background")
            .cast();

        Self {
            priority,
            size,

            tiles: vec![0; size.num_tiles()],
            is_dirty: true,

            scroll: Vector2D::default(),

            screenblock_ptr,

            transform: AffineMatrixBackground::default(),
            wrap_behaviour,
        }
    }

    pub fn set_scroll_pos(&mut self, scroll: impl Into<Vector2D<Num<i32, 8>>>) {
        self.scroll = scroll.into();
    }

    #[must_use]
    pub fn scroll_pos(&self) -> Vector2D<Num<i32, 8>> {
        self.scroll
    }

    pub fn set_tile(
        &mut self,
        pos: impl Into<Vector2D<i32>>,
        tileset: &TileSet<'_>,
        tile_index: u16,
    ) {
        assert_eq!(
            tileset.format(),
            TileFormat::EightBpp,
            "Can only use 256 colour tiles in an affine background"
        );

        let pos = self.size.gba_offset(pos.into());
        self.set_tile_at_pos(pos, tileset, tile_index);
    }

    pub fn set_transform(&mut self, transform: impl Into<AffineMatrixBackground>) {
        self.transform = transform.into();
    }

    fn set_tile_at_pos(&mut self, pos: usize, tileset: &TileSet<'_>, tile_index: u16) {
        let old_tile = self.tiles[pos];
        if old_tile != 0 {
            VRAM_MANAGER.remove_tile(TileIndex::EightBpp(old_tile as u16));
        }

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = VRAM_MANAGER.add_tile(tileset, tile_index);
            if new_tile_idx.raw_index() > u8::MAX as u16 {
                VRAM_MANAGER.remove_tile(new_tile_idx);
                0
            } else {
                new_tile_idx.raw_index() as u8
            }
        } else {
            0
        };

        if old_tile == new_tile {
            return;
        }

        self.tiles[pos] = new_tile;
        self.is_dirty = true;
    }

    pub fn commit(&mut self) {
        if self.is_dirty {
            unsafe {
                self.screenblock_ptr
                    .as_ptr()
                    .copy_from_nonoverlapping(self.tiles.as_ptr(), self.size.num_tiles());
            }
        }

        self.is_dirty = false;
    }

    pub fn clear(&mut self) {
        for tile in &mut self.tiles {
            if *tile != 0 {
                VRAM_MANAGER.remove_tile(TileIndex::EightBpp(*tile as u16));
            }

            *tile = 0;
        }

        self.is_dirty = true;
    }

    pub fn show(&self, frame: &mut GraphicsFrame<'_>) -> AffineBackgroundId {
        frame.bg_frame.set_next_affine(AffineBackgroundData {
            bg_ctrl: self.bg_ctrl(),
            scroll_offset: self.scroll,
            affine_transform: self.transform,
        })
    }

    fn bg_ctrl(&self) -> BackgroundControlRegister {
        let mut background_control_register = BackgroundControlRegister::default();

        background_control_register.set_priority(self.priority.into());
        background_control_register.set_screen_base_block(u5::new(self.screen_base_block() as u8));
        background_control_register.set_overflow_behaviour(self.wrap_behaviour.into());
        background_control_register.set_screen_size(self.size.into());

        background_control_register
    }

    fn screen_base_block(&self) -> u16 {
        let screenblock_location = self.screenblock_ptr.as_ptr() as usize;
        ((screenblock_location - VRAM_START) / SCREENBLOCK_SIZE) as u16
    }

    #[must_use]
    pub fn size(&self) -> AffineBackgroundSize {
        self.size
    }
}

impl Drop for AffineBackgroundTiles {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            ScreenblockAllocator.deallocate(self.screenblock_ptr.cast(), self.size.layout());
        }
    }
}
