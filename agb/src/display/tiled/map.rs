use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use crate::bitarray::Bitarray;
use crate::display::{object::AffineMatrixAttributes, Priority, DISPLAY_CONTROL};
use crate::dma::dma_copy16;
use crate::fixnum::{Num, Number, Vector2D};
use crate::memory_mapped::MemoryMapped;

use super::{
    AffineBackgroundSize, BackgroundID, BackgroundSize, BackgroundSizePrivate,
    RegularBackgroundSize, Tile, TileFormat, TileIndex, TileSet, TileSetting, VRamManager,
};

use crate::syscall::BgAffineSetData;
use alloc::{vec, vec::Vec};

pub(super) trait TiledMapPrivateConst: TiledMapTypes {
    type TileType: Into<TileIndex> + Copy + Default + Eq + PartialEq;
    type AffineMatrix;
    fn x_scroll(&self) -> Self::Position;
    fn y_scroll(&self) -> Self::Position;
    fn affine_matrix(&self) -> Self::AffineMatrix;
    fn background_id(&self) -> usize;
    fn screenblock(&self) -> usize;
    fn priority(&self) -> Priority;
    fn map_size(&self) -> Self::Size;
    fn bg_x(&self) -> MemoryMapped<Self::Position>;
    fn bg_y(&self) -> MemoryMapped<Self::Position>;
    fn bg_affine_matrix(&self) -> MemoryMapped<Self::AffineMatrix>;
    fn bg_control_register(&self) -> MemoryMapped<u16> {
        unsafe { MemoryMapped::new(0x0400_0008 + 2 * self.background_id()) }
    }
    fn screenblock_memory(&self) -> *mut u16 {
        (0x0600_0000 + 0x1000 * self.screenblock() as usize / 2) as *mut u16
    }
}

trait TiledMapPrivate: TiledMapPrivateConst {
    fn tiles_mut(&mut self) -> &mut [Self::TileType];
    fn tiles_dirty(&mut self) -> &mut bool;
    fn x_scroll_mut(&mut self) -> &mut Self::Position;
    fn y_scroll_mut(&mut self) -> &mut Self::Position;
}

pub trait TiledMapTypes {
    type Position: Number;
    type Size: BackgroundSize + Copy;
}

pub trait TiledMap: TiledMapTypes {
    fn clear(&mut self, vram: &mut VRamManager);
    fn show(&mut self);
    fn hide(&mut self);
    fn commit(&mut self, vram: &mut VRamManager);
    fn size(&self) -> Self::Size;

    #[must_use]
    fn scroll_pos(&self) -> Vector2D<Self::Position>;
    fn set_scroll_pos(&mut self, pos: Vector2D<Self::Position>);
}

impl<T> TiledMap for T
where
    T: TiledMapPrivateConst + TiledMapPrivate + TiledMapTypes,
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
        self.bg_x().set(self.x_scroll());
        self.bg_y().set(self.y_scroll());
        self.bg_affine_matrix().set(self.affine_matrix());

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

    fn size(&self) -> Self::Size {
        self.map_size()
    }

    #[must_use]
    fn scroll_pos(&self) -> Vector2D<Self::Position> {
        (self.x_scroll(), self.y_scroll()).into()
    }

    fn set_scroll_pos(&mut self, pos: Vector2D<Self::Position>) {
        *self.x_scroll_mut() = pos.x;
        *self.y_scroll_mut() = pos.y;
    }
}

pub struct RegularMap {
    background_id: u8,
    screenblock: u8,
    priority: Priority,
    size: RegularBackgroundSize,

    x_scroll: u16,
    y_scroll: u16,

    tiles: Vec<Tile>,
    tiles_dirty: bool,
}

pub const TRANSPARENT_TILE_INDEX: u16 = (1 << 10) - 1;

#[rustfmt::skip]
impl TiledMapPrivate for RegularMap {
    fn tiles_mut(&mut self) -> &mut [Self::TileType] { &mut self.tiles }
    fn tiles_dirty(&mut self) -> &mut bool { &mut self.tiles_dirty }
    fn x_scroll_mut(&mut self) -> &mut Self::Position { &mut self.x_scroll }
    fn y_scroll_mut(&mut self) -> &mut Self::Position { &mut self.y_scroll }
}

impl TiledMapTypes for RegularMap {
    type Position = u16;
    type Size = RegularBackgroundSize;
}

#[rustfmt::skip]
impl const TiledMapPrivateConst for RegularMap {
    type TileType = Tile;
    type AffineMatrix = ();
    fn x_scroll(&self) -> Self::Position { self.x_scroll }
    fn y_scroll(&self) -> Self::Position { self.y_scroll }
    fn affine_matrix(&self) -> Self::AffineMatrix {}
    fn background_id(&self) -> usize { self.background_id as usize }
    fn screenblock(&self) -> usize { self.screenblock as usize }
    fn priority(&self) -> Priority { self.priority }
    fn map_size(&self) -> Self::Size { self.size }
    fn bg_x(&self) -> MemoryMapped<Self::Position> {
        unsafe { MemoryMapped::new(0x0400_0010 + 4 * self.background_id as usize) }
    }
    fn bg_y(&self) -> MemoryMapped<Self::Position> {
        unsafe { MemoryMapped::new(0x0400_0012 + 4 * self.background_id as usize) }
    }
    fn bg_affine_matrix(&self) -> MemoryMapped<Self::AffineMatrix> {
        unsafe { MemoryMapped::new(0) }
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

            x_scroll: 0,
            y_scroll: 0,

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
}

pub struct AffineMap {
    background_id: u8,
    screenblock: u8,
    priority: Priority,
    size: AffineBackgroundSize,

    bg_center: Vector2D<Num<i32, 8>>,
    transform: BgAffineSetData,

    tiles: Vec<u8>,
    tiles_dirty: bool,
}

#[rustfmt::skip]
impl TiledMapPrivate for AffineMap {
    fn tiles_mut(&mut self) -> &mut [Self::TileType] { &mut self.tiles }
    fn tiles_dirty(&mut self) -> &mut bool { &mut self.tiles_dirty }
    fn x_scroll_mut(&mut self) -> &mut Self::Position { &mut self.transform.position.x }
    fn y_scroll_mut(&mut self) -> &mut Self::Position { &mut self.transform.position.y }
}

impl TiledMapTypes for AffineMap {
    type Position = Num<i32, 8>;
    type Size = AffineBackgroundSize;
}

#[rustfmt::skip]
impl const TiledMapPrivateConst for AffineMap {
    type TileType = u8;
    type AffineMatrix = AffineMatrixAttributes;
    fn x_scroll(&self) -> Self::Position { self.transform.position.x }
    fn y_scroll(&self) -> Self::Position { self.transform.position.y }
    fn affine_matrix(&self) -> Self::AffineMatrix { self.transform.matrix }
    fn background_id(&self) -> usize { self.background_id as usize }
    fn screenblock(&self) -> usize { self.screenblock as usize }
    fn priority(&self) -> Priority { self.priority }
    fn map_size(&self) -> Self::Size { self.size }
    fn bg_x(&self) -> MemoryMapped<Self::Position> {
        unsafe { MemoryMapped::new(0x0400_0008 + 0x10 * self.background_id()) }
    }
    fn bg_y(&self) -> MemoryMapped<Self::Position> {
        unsafe { MemoryMapped::new(0x0400_000c + 0x10 * self.background_id()) }
    }
    fn bg_affine_matrix(&self) -> MemoryMapped<Self::AffineMatrix> {
        unsafe { MemoryMapped::new(0x0400_0000 + 0x10 * self.background_id()) }
    }
}

impl AffineMap {
    pub(crate) fn new(
        background_id: u8,
        screenblock: u8,
        priority: Priority,
        size: AffineBackgroundSize,
        bg_center: Vector2D<Num<i32, 8>>,
    ) -> Self {
        Self {
            background_id,
            screenblock,
            priority,
            size,

            bg_center,
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

    pub fn set_transform_raw(&mut self, transform: BgAffineSetData) {
        self.transform = transform;
    }

    pub fn set_transform(
        &mut self,
        display_center: Vector2D<i16>,
        scale: Vector2D<Num<i16, 8>>,
        rotation: Num<u8, 8>,
    ) {
        self.set_transform_raw(crate::syscall::bg_affine_matrix(
            self.bg_center,
            display_center,
            scale,
            rotation,
        ));
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
