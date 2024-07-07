use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use crate::bitarray::Bitarray;
use crate::display::affine::AffineMatrixBackground;
use crate::display::tile_data::TileData;
use crate::display::{Priority, DISPLAY_CONTROL};
use crate::dma;
use crate::fixnum::Vector2D;
use crate::memory_mapped::MemoryMapped;

use super::{
    AffineBackgroundSize, BackgroundID, BackgroundSize, BackgroundSizePrivate,
    RegularBackgroundSize, Tile, TileFormat, TileSet, TileSetting, VRamManager,
};

use alloc::{vec, vec::Vec};
use crate::display::tiled::TileFormat::FourBpp;

pub trait TiledMapTypes: private::Sealed {
    type Size: BackgroundSize + Copy;
}

trait TiledMapPrivate: TiledMapTypes {
    type AffineMatrix;

    fn tiles_mut(&mut self) -> &mut [Tile];
    fn tiles_dirty(&mut self) -> &mut bool;

    fn colours(&self) -> TileFormat;

    fn background_id(&self) -> usize;
    fn screenblock(&self) -> usize;
    fn priority(&self) -> Priority;
    fn map_size(&self) -> Self::Size;

    fn update_bg_registers(&self);

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
    fn set_visible(&mut self, visible: bool);
    fn is_visible(&self) -> bool;
    fn commit(&mut self, vram: &mut VRamManager);
    fn size(&self) -> Self::Size;
}

impl TiledMap for AffineMap
{
    fn clear(&mut self, vram: &mut VRamManager) {
        let colours = self.colours();

        for tile in self.tiles_mut() {
            if *tile != Default::default() {
                vram.remove_tile(tile.tile_index(colours));
            }

            *tile = Default::default();
        }
    }

    /// Sets wether the map is visible  
    /// Use [is_visible](TiledMap::is_visible) to get the value
    fn set_visible(&mut self, visible: bool) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = if visible {
            mode | (1 << (self.background_id() + 0x08)) as u16
        } else {
            mode & !(1 << (self.background_id() + 0x08)) as u16
        };
        DISPLAY_CONTROL.set(new_mode);
    }

    /// Checks whether the map is not marked as hidden  
    /// Use [set_visible](TiledMap::set_visible) to set the value
    fn is_visible(&self) -> bool {
        DISPLAY_CONTROL.get() & (1 << (self.background_id() + 0x08)) > 0
    }

    fn commit(&mut self, vram: &mut VRamManager) {
        let screenblock_memory = self.screenblock_memory() as *mut u8;

        if *self.tiles_dirty() {
            unsafe {
                let tiledata: Vec<u8> = self.tiles_mut().iter().map(|a| a.tile_index(FourBpp).raw_index() as u8).collect();
                screenblock_memory.copy_from(
                    tiledata.as_ptr(),
                    self.map_size().num_tiles(),
                );
            }
        }

        let tile_colour_flag: u16 = (self.colours() == TileFormat::EightBpp).into();

        let new_bg_control_value = (self.priority() as u16)
            | ((self.screenblock() as u16) << 8)
            | (tile_colour_flag << 7)
            | (self.map_size().size_flag() << 14);

        self.bg_control_register().set(new_bg_control_value);
        self.update_bg_registers();

        vram.gc();

        *self.tiles_dirty() = false;
    }

    fn size(&self) -> <AffineMap as TiledMapTypes>::Size {
        self.map_size()
    }
}
impl TiledMap for RegularMap
{
    fn clear(&mut self, vram: &mut VRamManager) {
        let colours = self.colours();

        for tile in self.tiles_mut() {
            if *tile != Default::default() {
                vram.remove_tile(tile.tile_index(colours));
            }

            *tile = Default::default();
        }
    }

    /// Sets wether the map is visible
    /// Use [is_visible](TiledMap::is_visible) to get the value
    fn set_visible(&mut self, visible: bool) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = if visible {
            mode | (1 << (self.background_id() + 0x08)) as u16
        } else {
            mode & !(1 << (self.background_id() + 0x08)) as u16
        };
        DISPLAY_CONTROL.set(new_mode);
    }

    /// Checks whether the map is not marked as hidden
    /// Use [set_visible](TiledMap::set_visible) to set the value
    fn is_visible(&self) -> bool {
        DISPLAY_CONTROL.get() & (1 << (self.background_id() + 0x08)) > 0
    }

    fn commit(&mut self, vram: &mut VRamManager) {
        let screenblock_memory = self.screenblock_memory();

        if *self.tiles_dirty() {
            unsafe {
                screenblock_memory.copy_from(
                    self.tiles_mut().as_ptr() as *const u16,
                    self.map_size().num_tiles(),
                );
            }
        }

        let tile_colour_flag: u16 = (self.colours() == TileFormat::EightBpp).into();

        let new_bg_control_value = (self.priority() as u16)
            | ((self.screenblock() as u16) << 8)
            | (tile_colour_flag << 7)
            | (self.map_size().size_flag() << 14);

        self.bg_control_register().set(new_bg_control_value);
        self.update_bg_registers();

        vram.gc();

        *self.tiles_dirty() = false;
    }

    fn size(&self) -> <RegularMap as TiledMapTypes>::Size {
        self.map_size()
    }
}

pub struct RegularMap {
    background_id: u8,
    screenblock: u8,
    priority: Priority,
    size: RegularBackgroundSize,

    colours: TileFormat,

    scroll: Vector2D<i16>,

    tiles: Vec<Tile>,
    tiles_dirty: bool,
}

pub(crate) const TRANSPARENT_TILE_INDEX: u16 = (1 << 10) - 1;

impl TiledMapTypes for RegularMap {
    type Size = RegularBackgroundSize;
}

impl TiledMapPrivate for RegularMap {
    type AffineMatrix = ();

    fn tiles_mut(&mut self) -> &mut [Tile] {
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
    fn colours(&self) -> TileFormat {
        self.colours
    }
}

impl RegularMap {
    pub(crate) fn new(
        background_id: u8,
        screenblock: u8,
        priority: Priority,
        size: RegularBackgroundSize,
        colours: TileFormat,
    ) -> Self {
        Self {
            background_id,
            screenblock,
            priority,
            size,

            scroll: Default::default(),

            colours,

            tiles: vec![Default::default(); size.num_tiles()],
            tiles_dirty: true,
        }
    }

    pub fn fill_with(&mut self, vram: &mut VRamManager, tile_data: &TileData) {
        assert!(
            tile_data.tile_settings.len() >= 20 * 30,
            "Don't have a full screen's worth of tile data"
        );

        assert_eq!(
            tile_data.tiles.format(),
            self.colours(),
            "Cannot set a {:?} colour tile on a {:?} colour background",
            tile_data.tiles.format(),
            self.colours()
        );

        for y in 0..20 {
            for x in 0..30 {
                let tile_id = y * 30 + x;
                let tile_pos = y * 32 + x;
                self.set_tile_at_pos(
                    vram,
                    tile_pos,
                    &tile_data.tiles,
                    tile_data.tile_settings[tile_id],
                );
            }
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
            self.colours(),
            "Cannot set a {:?} colour tile on a {:?} colour background",
            tileset.format(),
            self.colours()
        );

        let pos = self.map_size().gba_offset(pos.into());
        self.set_tile_at_pos(vram, pos, tileset, tile_setting);
    }

    fn set_tile_at_pos(
        &mut self,
        vram: &mut VRamManager,
        pos: usize,
        tileset: &TileSet<'_>,
        tile_setting: TileSetting,
    ) {
        let old_tile = self.tiles_mut()[pos];
        if old_tile != Tile::default() {
            vram.remove_tile(old_tile.tile_index(self.colours()));
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

    /// Returns the latest map priority set  
    /// This will only be the currently applied priority if you called [commit](TiledMap::commit) before calling this function  
    /// Use [set_priority](Self::set_priority) to set the value
    #[must_use]
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Sets the map priority  
    /// This require to call [commit](TiledMap::commit) in order to apply the value  
    /// Use [priority](Self::priority) to get the value
    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    #[must_use]
    pub fn scroll_pos(&self) -> Vector2D<i16> {
        self.scroll
    }

    pub fn set_scroll_pos(&mut self, pos: impl Into<Vector2D<i16>>) {
        self.scroll = pos.into();
    }

    #[must_use]
    pub fn x_scroll_dma(&self) -> dma::DmaControllable<i16> {
        dma::DmaControllable::new(self.x_register().as_ptr())
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

    transform: AffineMatrixBackground,

    tiles: Vec<Tile>,
    tiles_dirty: bool,
}

impl TiledMapTypes for AffineMap {
    type Size = AffineBackgroundSize;
}

impl TiledMapPrivate for AffineMap {
    type AffineMatrix = AffineMatrixBackground;

    fn tiles_mut(&mut self) -> &mut [Tile] {
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
    fn colours(&self) -> TileFormat {
        TileFormat::EightBpp
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

            transform: Default::default(),

            tiles: vec![Default::default(); size.num_tiles()],
            tiles_dirty: true,
        }
    }

    pub fn set_tile(
        &mut self,
        vram: &mut VRamManager,
        pos: impl Into<Vector2D<u16>>,
        tileset: &TileSet<'_>,
        tile_id: u8,
    ) {
        let pos = self.map_size().gba_offset(pos.into());
        let colours = self.colours();

        let old_tile = self.tiles_mut()[pos];
        if old_tile != Tile::default() {
            vram.remove_tile(old_tile.tile_index(colours));
        }

        let tile_index = tile_id as u16;

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = vram.add_tile(tileset, tile_index);
            Tile::new(new_tile_idx, TileSetting(0))
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

    pub fn set_transform(&mut self, transformation: impl Into<AffineMatrixBackground>) {
        self.transform = transformation.into();
    }

    // Gets the map priority
    #[must_use]
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Sets the map priority
    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
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
