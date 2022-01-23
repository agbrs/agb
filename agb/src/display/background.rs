use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::memory_mapped::{MemoryMapped, MemoryMapped1DArray};

use super::{
    palette16, set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, Priority,
    DISPLAY_CONTROL,
};

const TILE_BACKGROUND: MemoryMapped1DArray<u32, { 2048 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06000000) };

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };

#[derive(Clone, Copy, Debug)]
pub enum TileFormat {
    FourBpp,
}

impl TileFormat {
    /// Returns the size of the tile in bytes
    fn tile_size(self) -> usize {
        match self {
            TileFormat::FourBpp => 8 * 8 / 2,
        }
    }
}

pub struct TileSet<'a> {
    tiles: &'a [u32],
    format: TileFormat,
}

impl<'a> TileSet<'a> {
    pub fn new(tiles: &'a [u32], format: TileFormat) -> Self {
        Self { tiles, format }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TileSetReference {
    id: u16,
    generation: u32,
}

#[derive(Debug)]
pub struct TileIndex(u16);

enum ArenaStorageItem<T> {
    EndOfFreeList,
    NextFree(usize),
    Data(T, u32),
}

pub struct VRamManager<'a> {
    tilesets: Vec<ArenaStorageItem<TileSet<'a>>>,
    generation: u32,
    free_pointer: Option<usize>,

    tile_set_to_vram: HashMap<(u16, u16), u16>,
    references: Vec<u16>,
    vram_free_pointer: Option<usize>,
}

const END_OF_FREE_LIST_MARKER: u16 = u16::MAX;

impl<'a> VRamManager<'a> {
    pub fn new() -> Self {
        Self {
            tilesets: Vec::new(),
            generation: 0,
            free_pointer: None,

            tile_set_to_vram: HashMap::new(),
            references: Vec::new(),
            vram_free_pointer: None,
        }
    }

    pub fn add_tileset(&mut self, tileset: TileSet<'a>) -> TileSetReference {
        let generation = self.generation;
        self.generation += 1;

        let tileset = ArenaStorageItem::Data(tileset, generation);

        let index = if let Some(ptr) = self.free_pointer.take() {
            match self.tilesets[ptr] {
                ArenaStorageItem::EndOfFreeList => {
                    self.tilesets[ptr] = tileset;
                    ptr
                }
                ArenaStorageItem::NextFree(next_free) => {
                    self.free_pointer = Some(next_free);
                    self.tilesets[ptr] = tileset;
                    ptr
                }
                _ => panic!("Free pointer shouldn't point to valid data"),
            }
        } else {
            self.tilesets.push(tileset);
            self.tilesets.len() - 1
        };

        TileSetReference::new(index as u16, generation)
    }

    pub fn remove_tileset(&mut self, tile_set_ref: TileSetReference) {
        let tileset = &self.tilesets[tile_set_ref.id as usize];

        match tileset {
            ArenaStorageItem::Data(_, generation) => {
                assert_eq!(
                    *generation, tile_set_ref.generation,
                    "Tileset generation must be the same when removing"
                );

                self.tilesets[tile_set_ref.id as usize] = if let Some(ptr) = self.free_pointer {
                    ArenaStorageItem::NextFree(ptr)
                } else {
                    ArenaStorageItem::EndOfFreeList
                };

                self.free_pointer = Some(tile_set_ref.id as usize);
            }
            _ => panic!("Already freed, probably a double free?"),
        }
    }

    pub fn add_tile(&mut self, tile_set_ref: TileSetReference, tile: u16) -> TileIndex {
        if let Some(&reference) = self.tile_set_to_vram.get(&(tile_set_ref.id, tile)) {
            self.references[reference as usize] += 1;
            return TileIndex(reference as u16);
        }

        let index_to_copy_into = if let Some(ptr) = self.vram_free_pointer.take() {
            if self.references[ptr] != END_OF_FREE_LIST_MARKER {
                self.vram_free_pointer = Some(self.references[ptr] as usize);
            }

            self.references[ptr] = 1;
            ptr
        } else {
            self.references.push(1);
            self.references.len() - 1
        };

        let tile_slice = if let ArenaStorageItem::Data(data, generation) =
            &self.tilesets[tile_set_ref.id as usize]
        {
            assert_eq!(
                *generation, tile_set_ref.generation,
                "Stale tile data requested"
            );

            let tile_offset = (tile as usize) * data.format.tile_size() / 4;
            &data.tiles[tile_offset..(tile_offset + data.format.tile_size() / 4)]
        } else {
            panic!("Cannot find tile data at given reference");
        };

        let tile_size_in_words = TileFormat::FourBpp.tile_size() / 4;

        for (i, &word) in tile_slice.iter().enumerate() {
            TILE_BACKGROUND.set(index_to_copy_into * tile_size_in_words + i, word);
        }

        TileIndex(index_to_copy_into as u16)
    }

    pub fn remove_tile(&mut self, tile_index: TileIndex) {
        let index = tile_index.0 as usize;
        self.references[index] -= 1;

        if self.references[index] != 0 {
            return;
        }

        if let Some(ptr) = self.vram_free_pointer {
            self.references[index] = ptr as u16;
        } else {
            self.references[index] = END_OF_FREE_LIST_MARKER;
        }

        self.vram_free_pointer = Some(index);
    }

    /// Copies raw palettes to the background palette without any checks.
    pub fn set_background_palette_raw(&mut self, palette: &[u16]) {
        for (index, &colour) in palette.iter().enumerate() {
            PALETTE_BACKGROUND.set(index, colour);
        }
    }

    fn set_background_palette(&mut self, pal_index: u8, palette: &palette16::Palette16) {
        for (colour_index, &colour) in palette.colours.iter().enumerate() {
            PALETTE_BACKGROUND.set(pal_index as usize * 16 + colour_index, colour);
        }
    }

    /// Copies palettes to the background palettes without any checks.
    pub fn set_background_palettes(&mut self, palettes: &[palette16::Palette16]) {
        for (palette_index, entry) in palettes.iter().enumerate() {
            self.set_background_palette(palette_index as u8, entry)
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Tile(u16);

impl Tile {
    pub fn new(tid: TileIndex, hflip: bool, vflip: bool, palette_id: u16) -> Self {
        Self(tid.0 | ((hflip as u16) << 10) | ((vflip as u16) << 11) | (palette_id << 12))
    }
}

pub struct RegularMap {
    background_id: u8,

    screenblock: u8,
    x_scroll: u16,
    y_scroll: u16,
    priority: Priority,

    tiles: [Tile; 32 * 32],
    tiles_dirty: bool,
}

impl RegularMap {
    fn new(background_id: u8, screenblock: u8) -> Self {
        Self {
            background_id,

            screenblock,
            x_scroll: 0,
            y_scroll: 0,
            priority: Priority::P0,

            tiles: [Tile(0); 32 * 32],
            tiles_dirty: true,
        }
    }

    pub fn set_tile(&mut self, x: u16, y: u16, tile: Tile) {
        self.tiles[(x + y * 32) as usize] = tile;
        self.tiles_dirty = true;
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
        for (i, tile) in self.tiles.iter().enumerate() {
            screenblock_memory.set(i, tile.0);
        }

        self.tiles_dirty = false;
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

pub struct Tiled0<'a> {
    num_regular: u8,
    next_screenblock: u8,

    pub vram: VRamManager<'a>,
}

impl Tiled0<'_> {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_settings(GraphicsSettings::empty() | GraphicsSettings::SPRITE1_D);
        set_graphics_mode(DisplayMode::Tiled0);

        Self {
            num_regular: 0,
            next_screenblock: 16,

            vram: VRamManager::new(),
        }
    }

    pub fn background(&mut self) -> RegularMap {
        if self.num_regular == 4 {
            panic!("Can only create 4 backgrounds");
        }

        let bg = RegularMap::new(self.num_regular, self.next_screenblock);

        self.num_regular += 1;
        self.next_screenblock += 1;

        bg
    }
}

impl TileSetReference {
    fn new(id: u16, generation: u32) -> Self {
        Self { id, generation }
    }
}
