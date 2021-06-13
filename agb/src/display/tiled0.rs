use core::{convert::TryInto, ops::Deref};

use crate::{
    memory_mapped::MemoryMapped1DArray,
    number::{Rect, Vector2D},
};

use super::{
    palette16, set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, Priority,
    DISPLAY_CONTROL,
};

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };
const PALETTE_SPRITE: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0200) };

const TILE_BACKGROUND: MemoryMapped1DArray<u32, { 2048 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06000000) };
const TILE_SPRITE: MemoryMapped1DArray<u32, { 512 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06010000) };

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

#[non_exhaustive]
/// The map background is the method of drawing game maps to the screen. It
/// automatically handles copying the correct portion of a provided map to the
/// assigned block depending on given coordinates.
pub struct Background {
    background: u8,
    block: u8,
    pos_x: i32,
    pos_y: i32,
}

impl Background {
    unsafe fn new(layer: u8, block: u8) -> Background {
        let mut background = Background {
            background: layer,
            block,
            pos_x: 0,
            pos_y: 0,
        };
        background.set_colour_mode(ColourMode::FourBitPerPixel);
        background.set_background_size(BackgroundSize::S32x32);
        background.set_block(block);
        background
    }
}

impl Background {
    /// Sets the background to be shown on screen. Requires the background to
    /// have a map enabled otherwise a panic is caused.
    pub fn show(&mut self) {
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

    unsafe fn get_register(&mut self) -> *mut u16 {
        (0x0400_0008 + 2 * self.background as usize) as *mut u16
    }

    unsafe fn set_block(&mut self, block: u8) {
        self.set_bits(0x08, 5, block as u16)
    }

    unsafe fn set_bits(&mut self, start: u16, num_bits: u16, bits: u16) {
        let reg = self.get_register();
        let control = reg.read_volatile();
        let mask = !(((1 << num_bits) - 1) << start);
        let new_control = (control & mask) | ((bits as u16) << start);
        reg.write_volatile(new_control);
    }

    /// Sets priority of the background layer. Backgrounds with higher priority
    /// are drawn (above/below) backgrounds with lower priority.
    pub fn set_priority(&mut self, p: Priority) {
        unsafe { self.set_bits(0, 2, p as u16) }
    }

    fn set_colour_mode(&mut self, mode: ColourMode) {
        unsafe { self.set_bits(0x07, 1, mode as u16) }
    }

    fn set_background_size(&mut self, size: BackgroundSize) {
        unsafe { self.set_bits(0x0E, 2, size as u16) }
    }

    fn map_get<T>(&self, map: &T, dim_x: u32, dim_y: u32, x: i32, y: i32, default: u16) -> u16
    where
        T: Deref<Target = [u16]>,
    {
        if x >= dim_x as i32 || x < 0 || y >= dim_y as i32 || y < 0 {
            default
        } else {
            map[(dim_x as i32 * y + x) as usize]
        }
    }

    fn set_x(&self, x: u16) {
        unsafe { *((0x0400_0010 + 4 * self.background as usize) as *mut u16) = x }
    }
    fn set_y(&self, y: u16) {
        unsafe { *((0x0400_0012 + 4 * self.background as usize) as *mut u16) = y }
    }

    /// Forces the portion of the map in current view to be copied to the map
    /// block assigned to this background. This is currently unnecesary to call.
    /// Setting position already updates the drawn map, and changing map forces
    /// an update.
    pub fn draw_full_map(&mut self, map: &[u16], dimensions: Vector2D<u32>, default: u16) {
        let area: Rect<i32> = Rect {
            position: Vector2D::new(-1, -1),
            size: Vector2D::new(32, 22),
        };
        self.draw_area(map, dimensions, area, default);
    }

    /// Forces a specific area of the screen to be drawn, taking into account any positonal offsets.
    pub fn draw_area(&self, map: &[u16], dimensions: Vector2D<u32>, area: Rect<i32>, default: u16) {
        self.draw_area_mapped(
            &map,
            dimensions.x,
            dimensions.y,
            area.position.x,
            area.position.y,
            area.size.x,
            area.size.y,
            default,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_area_mapped<T>(
        &self,
        map: &T,
        dim_x: u32,
        dim_y: u32,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
        default: u16,
    ) where
        T: Deref<Target = [u16]>,
    {
        let x_map_space = self.pos_x / 8;
        let y_map_space = self.pos_y / 8;

        let x_block_space = x_map_space % 32;
        let y_block_space = y_map_space % 32;

        for x in left..(left + width) {
            for y in top..(top + height) {
                unsafe {
                    (&mut (*MAP)[self.block as usize][(y_block_space + y).rem_euclid(32) as usize]
                        [(x_block_space + x).rem_euclid(32) as usize]
                        as *mut u16)
                        .write_volatile(self.map_get(
                            map,
                            dim_x,
                            dim_y,
                            x_map_space + x,
                            y_map_space + y,
                            default,
                        ))
                };
            }
        }
    }

    /// Sets the position of the map to be shown on screen. This automatically
    /// manages copying the correct portion to the map block and moving the map
    /// registers.
    pub fn set_position(
        &mut self,
        map: &[u16],
        dimensions: Vector2D<u32>,
        position: Vector2D<i32>,
        default: u16,
    ) {
        self.set_position_mapped(
            &map,
            dimensions.x,
            dimensions.y,
            position.x,
            position.y,
            default,
        );
    }

    fn set_position_mapped<T>(
        &mut self,
        map: &T,
        dim_x: u32,
        dim_y: u32,
        x: i32,
        y: i32,
        default: u16,
    ) where
        T: Deref<Target = [u16]>,
    {
        let x_map_space = x / 8;
        let y_map_space = y / 8;

        let prev_x_map_space = self.pos_x / 8;
        let prev_y_map_space = self.pos_y / 8;

        let x_difference = x_map_space - prev_x_map_space;
        let y_difference = y_map_space - prev_y_map_space;

        let x_block_space = x_map_space % 32;
        let y_block_space = y_map_space % 32;

        self.pos_x = x;
        self.pos_y = y;

        // don't fancily handle if we've moved more than one tile, just copy the whole new map
        if x_difference.abs() > 1 || y_difference.abs() > 1 {
            self.draw_area_mapped(map, dim_x, dim_y, -1, -1, 32, 22, default);
        } else {
            if x_difference != 0 {
                let x_offset = match x_difference {
                    -1 => -1,
                    1 => 30,
                    _ => unreachable!(),
                };
                for y in -1..21 {
                    unsafe {
                        (&mut (*MAP)[self.block as usize]
                            [(y_block_space + y).rem_euclid(32) as usize]
                            [(x_block_space + x_offset).rem_euclid(32) as usize]
                            as *mut u16)
                            .write_volatile(self.map_get(
                                map,
                                dim_x,
                                dim_y,
                                x_map_space + x_offset,
                                y_map_space + y,
                                default,
                            ))
                    };
                }
            }
            if y_difference != 0 {
                let y_offset = match y_difference {
                    -1 => -1,
                    1 => 20,
                    _ => unreachable!(),
                };
                for x in -1..31 {
                    unsafe {
                        (&mut (*MAP)[self.block as usize]
                            [(y_block_space + y_offset).rem_euclid(32) as usize]
                            [(x_block_space + x).rem_euclid(32) as usize]
                            as *mut u16)
                            .write_volatile(self.map_get(
                                map,
                                dim_x,
                                dim_y,
                                x_map_space + x,
                                y_map_space + y_offset,
                                default,
                            ))
                    };
                }
            }
        }

        let x_remainder = x % (32 * 8);
        let y_remainder = y % (32 * 8);

        self.set_x(x_remainder as u16);
        self.set_y(y_remainder as u16);
    }
}

#[non_exhaustive]
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

    fn set_sprite_palette_entry(&mut self, index: u8, colour: u16) {
        PALETTE_SPRITE.set(index as usize, colour)
    }
    fn set_sprite_tilemap_entry(&mut self, index: u32, data: u32) {
        TILE_SPRITE.set(index as usize, data);
    }

    fn set_background_tilemap_entry(&mut self, index: u32, data: u32) {
        TILE_BACKGROUND.set(index as usize, data);
    }

    /// Copies raw palettes to the background palette without any checks.
    pub fn set_sprite_palette_raw(&mut self, colour: &[u16]) {
        for (index, &entry) in colour.iter().enumerate() {
            self.set_sprite_palette_entry(index.try_into().unwrap(), entry)
        }
    }
    fn set_sprite_palette(&mut self, pal_index: u8, palette: &palette16::Palette16) {
        for (colour_index, &colour) in palette.colours.iter().enumerate() {
            PALETTE_SPRITE.set(pal_index as usize * 16 + colour_index, colour);
        }
    }

    pub fn set_sprite_palettes(&mut self, palettes: &[palette16::Palette16]) {
        for (palette_index, entry) in palettes.iter().enumerate() {
            self.set_sprite_palette(palette_index as u8, entry)
        }
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

    /// Copies tiles to the sprite tilemap without any checks.
    pub fn set_sprite_tilemap(&mut self, tiles: &[u32]) {
        for (index, &tile) in tiles.iter().enumerate() {
            self.set_sprite_tilemap_entry(index as u32, tile)
        }
    }

    /// Gets a map background if possible and assigns an unused block to it.
    pub fn get_background(&mut self) -> Result<Background, &'static str> {
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
