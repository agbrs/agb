use core::ops::{Deref, Index};

use crate::{
    memory_mapped::{MemoryMapped, MemoryMapped1DArray},
    number::{Rect, Vector2D},
};

use super::{
    palette16, set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, Priority,
    DISPLAY_CONTROL,
};

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };

const TILE_BACKGROUND: MemoryMapped1DArray<u32, { 2048 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06000000) };

const MAP: *mut [[[u16; 32]; 32]; 32] = 0x0600_0000 as *mut _;

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

pub trait MapStorage: Deref<Target = [u16]> {}
impl MapStorage for &[u16] {}
impl MapStorage for &mut [u16] {}

/// The map background is the method of drawing game maps to the screen. It
/// automatically handles copying the correct portion of a provided map to the
/// assigned block depending on given coordinates.
pub struct Background<S: MapStorage> {
    background: u8,
    block: u8,
    commited_position: Vector2D<i32>,
    shadowed_position: Vector2D<i32>,
    poisoned: bool,
    shadowed_register: u16,
    map: Option<Map<S>>,
}

pub struct Map<S: MapStorage> {
    pub store: S,
    pub dimensions: Vector2D<u32>,
    pub default: u16,
}

impl<'a, S: MapStorage> Map<S> {
    fn get_position(&self, x: i32, y: i32) -> u16 {
        if x < 0 || x as u32 >= self.dimensions.x {
            self.default
        } else if y < 0 || y as u32 >= self.dimensions.y {
            self.default
        } else {
            self.store[y as usize * self.dimensions.x as usize + x as usize]
        }
    }
}

impl<'a, S: MapStorage> Background<S> {
    unsafe fn new(background: u8, block: u8) -> Background<S> {
        let mut b = Background {
            background,
            block,
            commited_position: (0, 0).into(),
            shadowed_position: (0, 0).into(),
            shadowed_register: 0,
            poisoned: true,
            map: None,
        };
        b.set_block(block);
        b.set_colour_mode(ColourMode::FourBitPerPixel);
        b.set_background_size(BackgroundSize::S32x32);
        b
    }

    /// Sets the background to be shown on screen. Requires the background to
    /// have a map enabled otherwise a panic is caused.
    pub fn show(&mut self) {
        assert!(self.map.is_some());
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode | (1 << (self.background + 0x08));
        DISPLAY_CONTROL.set(new_mode);
    }

    /// Hides the background, nothing from this background is rendered to screen.
    pub fn hide(&mut self) {
        let mode = DISPLAY_CONTROL.get();
        let new_mode = mode & !(1 << (self.background + 0x08));
        DISPLAY_CONTROL.set(new_mode);
    }

    unsafe fn set_shadowed_register_bits(&mut self, value: u16, length: u16, shift: u16) {
        let mask = !(((1 << length) - 1) << shift);
        let new = (self.shadowed_register & mask) | (value << shift);
        self.shadowed_register = new;
    }

    unsafe fn get_register(&self) -> MemoryMapped<u16> {
        MemoryMapped::new(0x0400_0008 + 2 * self.background as usize)
    }

    unsafe fn set_block(&mut self, block: u8) {
        self.set_shadowed_register_bits(block as u16, 5, 0x8);
    }

    unsafe fn set_colour_mode(&mut self, mode: ColourMode) {
        self.set_shadowed_register_bits(mode as u16, 0x1, 0x7);
    }

    pub fn set_priority(&mut self, p: Priority) {
        unsafe { self.set_shadowed_register_bits(p as u16, 0x2, 0x0) };
    }

    unsafe fn set_background_size(&mut self, size: BackgroundSize) {
        self.set_shadowed_register_bits(size as u16, 0x2, 0xE);
    }

    unsafe fn set_position_x_register(&self, x: u16) {
        *((0x0400_0010 + 4 * self.background as usize) as *mut u16) = x
    }
    unsafe fn set_position_y_register(&self, y: u16) {
        *((0x0400_0012 + 4 * self.background as usize) as *mut u16) = y
    }

    pub fn set_position(&mut self, position: Vector2D<i32>) {
        self.shadowed_position = position;
    }

    pub fn get_map(&mut self) -> Option<&mut Map<S>> {
        self.poisoned = true;
        self.map.as_mut()
    }

    pub fn set_map(&mut self, map: Map<S>) {
        self.poisoned = true;
        self.map = Some(map);
    }

    pub fn commit_area(&mut self, area: Rect<i32>) {
        // commit shadowed register
        unsafe { self.get_register().set(self.shadowed_register) };

        let positions_to_be_updated = if self.poisoned {
            area.iter()
                .chain(Rect::new((0, 0).into(), (0, 0).into()).iter())
        } else {
            // calculate difference in positions
            let position_difference = self.shadowed_position - self.commited_position;
            let tile_position_difference = position_difference / 8;

            // how much of the map needs updating
            let difference_x: Rect<i32> = if tile_position_difference.x == 0 {
                Rect::new((0, 0).into(), (0, 0).into())
            } else if tile_position_difference.x > 0 {
                Rect::new((0, 0).into(), (tile_position_difference.x, 0).into())
            } else if tile_position_difference.x < 0 {
                Rect::new(
                    (32 + tile_position_difference.x, 0).into(),
                    (tile_position_difference.x.abs(), 0).into(),
                )
            } else {
                unreachable!();
            };

            let difference_y: Rect<i32> = if tile_position_difference.y == 0 {
                Rect::new((0, 0).into(), (0, 0).into())
            } else if tile_position_difference.y > 0 {
                Rect::new((0, 0).into(), (0, tile_position_difference.y).into())
            } else if tile_position_difference.y < 0 {
                Rect::new(
                    (0, 32 + tile_position_difference.y).into(),
                    (0, tile_position_difference.y.abs()).into(),
                )
            } else {
                unreachable!();
            };

            // update those positions

            let y_update = area.overlapping_rect(difference_y);
            let x_update = area.overlapping_rect(difference_x);

            y_update.iter().chain(x_update.iter())
        };

        if let Some(map) = &self.map {
            for (x, y) in positions_to_be_updated {
                let tile_space_position = self.shadowed_position / 8;
                unsafe {
                    (&mut (*MAP)[self.block as usize][y.rem_euclid(32) as usize]
                        [x.rem_euclid(32) as usize] as *mut u16)
                        .write_volatile(
                            map.get_position(x + tile_space_position.x, y + tile_space_position.y),
                        );
                }
            }
        }

        // update commited position

        self.commited_position = self.shadowed_position;

        // update position in registers

        unsafe {
            self.set_position_x_register((self.commited_position.x % (32 * 8)) as u16);
            self.set_position_y_register((self.commited_position.y % (32 * 8)) as u16);
        }
    }

    pub fn commit(&mut self) {
        let area: Rect<i32> = Rect {
            position: Vector2D::new(-1, -1),
            size: Vector2D::new(32, 22),
        };
        self.commit_area(area)
    }
}

pub struct Tiled0 {
    used_blocks: u32,
    num_backgrounds: u8,
}

impl Tiled0 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_settings(GraphicsSettings::empty() | GraphicsSettings::SPRITE1_D);
        set_graphics_mode(DisplayMode::Tiled0);
        Tiled0 {
            used_blocks: 0,
            num_backgrounds: 0,
        }
    }

    fn set_background_tilemap_entry(&mut self, index: u32, data: u32) {
        TILE_BACKGROUND.set(index as usize, data);
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

    /// Gets a map background if possible and assigns an unused block to it.
    pub fn get_background<S: MapStorage>(&mut self) -> Result<Background<S>, &'static str> {
        if self.num_backgrounds >= 4 {
            return Err("too many backgrounds created, maximum is 4");
        }

        if !self.used_blocks == 0 {
            return Err("all blocks are used");
        }

        let mut availiable_block = u8::MAX;

        for i in 0..32 {
            if (1 << i) & self.used_blocks == 0 {
                availiable_block = i;
                break;
            }
        }

        assert!(
            availiable_block != u8::MAX,
            "should be able to find a block"
        );

        self.used_blocks |= 1 << availiable_block;

        let background = self.num_backgrounds;
        self.num_backgrounds = background + 1;
        Ok(unsafe { Background::new(background, availiable_block) })
    }

    /// Copies tiles to tilemap starting at the starting tile. Cannot overwrite
    /// blocks that are already written to, panic is caused if this is attempted.
    pub fn set_background_tilemap(&mut self, start_tile: u32, tiles: &[u32]) {
        let u32_per_block = 512;

        let start_block = (start_tile * 8) / u32_per_block;
        // round up rather than down
        let end_block = (start_tile * 8 + tiles.len() as u32 + u32_per_block - 1) / u32_per_block;

        let blocks_to_use: u32 = ((1 << (end_block - start_block)) - 1) << start_block;

        assert!(
            self.used_blocks & blocks_to_use == 0,
            "blocks {} to {} should be unused for this copy to succeed",
            start_block,
            end_block
        );

        self.used_blocks |= blocks_to_use;

        for (index, &tile) in tiles.iter().enumerate() {
            self.set_background_tilemap_entry(start_tile * 8 + index as u32, tile)
        }
    }
}
