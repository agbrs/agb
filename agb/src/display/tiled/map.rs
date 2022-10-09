use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use crate::bitarray::Bitarray;
use crate::display::affine::AffineMatrixBackground;
use crate::display::{Priority, DISPLAY_CONTROL};
use crate::dma::dma_copy16;
use crate::fixnum::{Num, Vector2D};
use crate::memory_mapped::MemoryMapped;

use super::{
    AffineBackgroundSize, BackgroundID, BackgroundSize, BackgroundSizePrivate,
    RegularBackgroundSize, Tile, TileFormat, TileIndex, TileSet, TileSetting, VRamManager,
};

use alloc::{vec, vec::Vec};

pub trait TiledMapTypes: private::Sealed {
    type Size: BackgroundSize + Copy;
}

trait TiledMapPrivate: TiledMapTypes {
    type TileType: Into<TileIndex> + Copy + Default + Eq + PartialEq;
    type AffineMatrix;

    fn tiles_mut(&mut self) -> &mut [Self::TileType];
    fn tiles_dirty(&mut self) -> &mut bool;

    fn background_id(&self) -> usize;
    fn screenblock(&self) -> usize;
    fn priority(&self) -> Priority;
    fn map_size(&self) -> Self::Size;

    fn update_bg_registers(&self);

    fn scroll_pos(&self) -> Vector2D<i16>;
    fn set_scroll_pos(&mut self, new_pos: Vector2D<i16>);

    fn bg_control_register(&self) -> MemoryMapped<u16> {
        unsafe { MemoryMapped::new(0x0400_0008 + 2 * self.background_id()) }
    }
    fn screenblock_memory(&self) -> *mut u16 {
        (0x0600_0000 + 0x1000 * self.screenblock() / 2) as *mut u16
    }
}

/// Trait which describes methods available on both tiled maps and affine maps. Note that
/// it is 'sealed' so you cannot implement this yourself.
pub trait TiledMap: TiledMapTypes {
    fn clear(&mut self, vram: &mut VRamManager);
    fn show(&mut self);
    fn hide(&mut self);
    fn commit(&mut self, vram: &mut VRamManager);
    fn size(&self) -> Self::Size;

    #[must_use]
    fn scroll_pos(&self) -> Vector2D<i16>;
    fn set_scroll_pos(&mut self, pos: Vector2D<i16>);
}

impl<T> TiledMap for T
where
    T: TiledMapPrivate,
    T::Size: BackgroundSizePrivate,
{
    fn clear(&mut self, vram: &mut VRamManager) {
        for tile in self.tiles_mut() {
            if *tile != Default::default() {
                vram.remove_tile((*tile).into());
            }

            *tile = Default::default();
        }
    }

    fn show(&mut self) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode | (1 << (self.background_id() + 0x08)) as u16;
        DISPLAY_CONTROL.set(new_mode);
    }

    fn hide(&mut self) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode & !(1 << (self.background_id() + 0x08)) as u16;
        DISPLAY_CONTROL.set(new_mode);
    }

    fn commit(&mut self, vram: &mut VRamManager) {
        let new_bg_control_value = (self.priority() as u16)
            | ((self.screenblock() as u16) << 8)
            | (self.map_size().size_flag() << 14);

        self.bg_control_register().set(new_bg_control_value);
        self.update_bg_registers();

        let screenblock_memory = self.screenblock_memory();
        let x: TileIndex = unsafe { *self.tiles_mut().get_unchecked(0) }.into();
        let x = x.format().tile_size() / TileFormat::FourBpp.tile_size();
        if *self.tiles_dirty() {
            unsafe {
                dma_copy16(
                    self.tiles_mut().as_ptr() as *const u16,
                    screenblock_memory,
                    self.map_size().num_tiles() / x,
                );
            }
        }

        vram.gc();

        *self.tiles_dirty() = false;
    }

    fn size(&self) -> T::Size {
        self.map_size()
    }

    #[must_use]
    fn scroll_pos(&self) -> Vector2D<i16> {
        TiledMapPrivate::scroll_pos(self)
    }

    fn set_scroll_pos(&mut self, pos: Vector2D<i16>) {
        TiledMapPrivate::set_scroll_pos(self, pos);
    }
}

pub struct RegularMap {
    background_id: u8,
    screenblock: u8,
    priority: Priority,
    size: RegularBackgroundSize,

    scroll: Vector2D<i16>,

    tiles: Vec<Tile>,
    tiles_dirty: bool,
}

pub const TRANSPARENT_TILE_INDEX: u16 = (1 << 10) - 1;

impl TiledMapTypes for RegularMap {
    type Size = RegularBackgroundSize;
}

impl TiledMapPrivate for RegularMap {
    type TileType = Tile;
    type AffineMatrix = ();

    fn tiles_mut(&mut self) -> &mut [Self::TileType] {
        &mut self.tiles
    }
    fn tiles_dirty(&mut self) -> &mut bool {
        &mut self.tiles_dirty
    }

    fn background_id(&self) -> usize {
        self.background_id as usize
    }
    fn screenblock(&self) -> usize {
        self.screenblock as usize
    }
    fn priority(&self) -> Priority {
        self.priority
    }
    fn map_size(&self) -> Self::Size {
        self.size
    }
    fn update_bg_registers(&self) {
        self.x_register().set(self.scroll.x);
        self.y_register().set(self.scroll.y);
    }
    fn scroll_pos(&self) -> Vector2D<i16> {
        self.scroll
    }
    fn set_scroll_pos(&mut self, new_pos: Vector2D<i16>) {
        self.scroll = new_pos;
    }
}

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
            priority,
            size,

            scroll: Default::default(),

            tiles: vec![Default::default(); size.num_tiles()],
            tiles_dirty: true,
        }
    }

    pub fn set_tile(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<u16>,
        tileset: &TileSet<'_>,
        tile_setting: TileSetting,
    ) {
        let pos = self.map_size().gba_offset(pos);

        let old_tile = self.tiles_mut()[pos];
        if old_tile != Tile::default() {
            vram.remove_tile(old_tile.into());
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

        self.tiles_mut()[pos] = new_tile;
        *self.tiles_dirty() = true;
    }

    fn x_register(&self) -> MemoryMapped<i16> {
        unsafe { MemoryMapped::new(0x0400_0010 + 4 * self.background_id as usize) }
    }

    fn y_register(&self) -> MemoryMapped<i16> {
        unsafe { MemoryMapped::new(0x0400_0012 + 4 * self.background_id as usize) }
    }
}

pub struct AffineMap {
    background_id: u8,
    screenblock: u8,
    priority: Priority,
    size: AffineBackgroundSize,

    scroll: Vector2D<i16>,

    transform: AffineMatrixBackground,

    tiles: Vec<u8>,
    tiles_dirty: bool,
}

impl TiledMapTypes for AffineMap {
    type Size = AffineBackgroundSize;
}

impl TiledMapPrivate for AffineMap {
    type TileType = u8;
    type AffineMatrix = AffineMatrixBackground;

    fn tiles_mut(&mut self) -> &mut [Self::TileType] {
        &mut self.tiles
    }
    fn tiles_dirty(&mut self) -> &mut bool {
        &mut self.tiles_dirty
    }
    fn background_id(&self) -> usize {
        self.background_id as usize
    }
    fn screenblock(&self) -> usize {
        self.screenblock as usize
    }
    fn priority(&self) -> Priority {
        self.priority
    }
    fn map_size(&self) -> Self::Size {
        self.size
    }
    fn update_bg_registers(&self) {
        self.bg_affine_matrix().set(self.transform);
    }
    fn scroll_pos(&self) -> Vector2D<i16> {
        self.scroll
    }
    fn set_scroll_pos(&mut self, new_pos: Vector2D<i16>) {
        self.scroll = new_pos;
    }
}

impl AffineMap {
    pub(crate) fn new(
        background_id: u8,
        screenblock: u8,
        priority: Priority,
        size: AffineBackgroundSize,
    ) -> Self {
        Self {
            background_id,
            screenblock,
            priority,
            size,

            scroll: Default::default(),

            transform: Default::default(),

            tiles: vec![Default::default(); size.num_tiles()],
            tiles_dirty: true,
        }
    }

    pub fn set_tile(
        &mut self,
        vram: &mut VRamManager,
        pos: Vector2D<u16>,
        tileset: &TileSet<'_>,
        tile_id: u8,
    ) {
        let pos = self.map_size().gba_offset(pos);

        let old_tile = self.tiles_mut()[pos];
        if old_tile != 0 {
            vram.remove_tile(old_tile.into());
        }

        let tile_index = tile_id as u16;

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = vram.add_tile(tileset, tile_index);
            new_tile_idx.raw_index() as u8
        } else {
            0
        };

        if old_tile == new_tile {
            // no need to mark as dirty if nothing changes
            return;
        }

        self.tiles_mut()[pos] = new_tile;
        *self.tiles_dirty() = true;
    }

    pub fn set_transform(&mut self, transformation: impl Into<AffineMatrixBackground>) {
        self.transform = transformation.into();
    }

    fn bg_affine_matrix(&self) -> MemoryMapped<AffineMatrixBackground> {
        unsafe { MemoryMapped::new(0x0400_0000 + 0x10 * self.background_id()) }
    }
}

pub struct MapLoan<'a, T> {
    map: T,
    background_id: u8,
    screenblock_id: u8,
    screenblock_length: u8,
    map_list: &'a RefCell<Bitarray<1>>,
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
        map_list: &'a RefCell<Bitarray<1>>,
        screenblock_list: &'a RefCell<Bitarray<1>>,
    ) -> Self {
        MapLoan {
            map,
            background_id,
            screenblock_id,
            screenblock_length,
            map_list,
            screenblock_list,
        }
    }

    #[must_use]
    pub const fn background(&self) -> BackgroundID {
        BackgroundID(self.background_id)
    }
}

impl<'a, T> Drop for MapLoan<'a, T> {
    fn drop(&mut self) {
        self.map_list
            .borrow_mut()
            .set(self.background_id as usize, false);

        let mut screenblock_list = self.screenblock_list.borrow_mut();

        for i in self.screenblock_id..self.screenblock_id + self.screenblock_length {
            screenblock_list.set(i as usize, false);
        }
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::RegularMap {}
    impl Sealed for super::AffineMap {}
}
