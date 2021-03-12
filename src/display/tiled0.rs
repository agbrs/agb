use core::convert::TryInto;

use crate::memory_mapped::MemoryMapped1DArray;

use super::{
    object::Object, set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings,
    DISPLAY_CONTROL,
};

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };
const PALETTE_SPRITE: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0200) };

const TILE_BACKGROUND: MemoryMapped1DArray<u32, { 512 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06000000) };
const TILE_SPRITE: MemoryMapped1DArray<u32, { 512 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06010000) };

const MAP: *mut [[[u16; 32]; 32]; 32] = 0x0600_0000 as *mut _;

pub enum BackgroundLayer {
    Background0 = 0,
    Background1 = 1,
    Background2 = 2,
    Background3 = 3,
}

pub enum Prioriry {
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}

pub enum ColourMode {
    FourBitPerPixel = 0,
    EightBitPerPixel = 1,
}

pub enum BackgroundSize {
    S32x32 = 0,
    S64x32 = 1,
    S32x64 = 2,
    S64x64 = 3,
}

#[non_exhaustive]
pub struct Background {
    layer: usize,
}

impl Background {
    pub fn enable(&mut self) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode | (1 << (self.layer + 0x08));
        DISPLAY_CONTROL.set(new_mode);
    }

    pub fn disable(&mut self) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode | !(1 << (self.layer + 0x08));
        DISPLAY_CONTROL.set(new_mode);
    }

    unsafe fn get_register(&mut self) -> *mut u16 {
        (0x0400_0008 + 2 * self.layer) as *mut u16
    }

    unsafe fn set_bits(&mut self, start: u16, num_bits: u16, bits: u16) {
        let reg = self.get_register();
        let control = reg.read_volatile();
        let mask = !(((1 << num_bits) - 1) << start);
        let new_control = (control & mask) | ((bits as u16) << start);
        reg.write_volatile(new_control);
    }

    pub fn set_priority(&mut self, p: Prioriry) {
        unsafe { self.set_bits(0, 2, p as u16) }
    }

    pub fn set_colour_mode(&mut self, mode: ColourMode) {
        unsafe { self.set_bits(0x07, 1, mode as u16) }
    }

    pub fn set_background_size(&mut self, size: BackgroundSize) {
        unsafe { self.set_bits(0x0E, 2, size as u16) }
    }

    pub fn set_screen_base_block(&mut self, block: u32) {
        assert!(
            block < 32,
            "screen base block must be in range 0 to 31 inclusive"
        );
        unsafe { self.set_bits(0x08, 5, block as u16) }
    }
}

#[non_exhaustive]
pub struct Tiled0 {
    pub background_0: Background,
    pub background_1: Background,
    pub background_2: Background,
    pub background_3: Background,
    pub object: Object,
}

impl Tiled0 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_settings(GraphicsSettings::empty());
        set_graphics_mode(DisplayMode::Tiled0);
        Tiled0 {
            background_0: Background { layer: 0 },
            background_1: Background { layer: 1 },
            background_2: Background { layer: 2 },
            background_3: Background { layer: 3 },
            object: Object::new(),
        }
    }

    pub fn set_sprite_palette_entry(&mut self, index: u8, colour: u16) {
        PALETTE_SPRITE.set(index as usize, colour)
    }
    pub fn set_background_palette_entry(&mut self, index: u8, colour: u16) {
        PALETTE_BACKGROUND.set(index as usize, colour)
    }
    pub fn set_sprite_tilemap_entry(&mut self, index: u32, data: u32) {
        TILE_SPRITE.set(index as usize, data);
    }

    pub fn set_background_tilemap_entry(&mut self, index: u32, data: u32) {
        TILE_BACKGROUND.set(index as usize, data);
    }

    pub fn set_sprite_palette(&mut self, colour: &[u16]) {
        for (index, &entry) in colour.iter().enumerate() {
            self.set_sprite_palette_entry(index.try_into().unwrap(), entry)
        }
    }

    pub fn set_background_palette(&mut self, colour: &[u16]) {
        for (index, &entry) in colour.iter().enumerate() {
            self.set_background_palette_entry(index.try_into().unwrap(), entry)
        }
    }

    pub fn set_sprite_tilemap(&mut self, tiles: &[u32]) {
        for (index, &tile) in tiles.iter().enumerate() {
            self.set_sprite_tilemap_entry(index as u32, tile)
        }
    }

    pub fn set_background_tilemap(&mut self, tiles: &[u32]) {
        for (index, &tile) in tiles.iter().enumerate() {
            self.set_background_tilemap_entry(index as u32, tile)
        }
    }

    pub fn copy_to_map(&mut self, map_id: usize, entries: &[u16]) {
        let map =
            unsafe { &mut ((*(MAP as *mut [[u16; 32 * 32]; 32]))[map_id]) as *mut [u16; 32 * 32] };
        for (index, &entry) in entries.iter().enumerate() {
            unsafe { (&mut (*map)[index] as *mut u16).write_volatile(entry) }
        }
    }
}
