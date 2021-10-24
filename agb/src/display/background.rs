use core::ops::Index;

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

#[derive(Clone, Copy)]
pub enum BackgroundSize {
    S32x32 = 0,
    S64x32 = 1,
    S32x64 = 2,
    S64x64 = 3,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Mutability {
    Immutable,
    Mutable,
}

struct MapStorage<'a> {
    s: *const [u16],
    mutability: Mutability,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> Index<usize> for MapStorage<'a> {
    type Output = u16;
    fn index(&self, index: usize) -> &Self::Output {
        &self.get()[index]
    }
}

impl<'a> MapStorage<'a> {
    fn new(store: &[u16]) -> MapStorage {
        MapStorage {
            s: store as *const _,
            mutability: Mutability::Immutable,
            _phantom: core::marker::PhantomData,
        }
    }
    fn new_mutable(store: &mut [u16]) -> MapStorage {
        MapStorage {
            s: store as *const _,
            mutability: Mutability::Mutable,
            _phantom: core::marker::PhantomData,
        }
    }
    fn get(&self) -> &[u16] {
        unsafe { &*self.s }
    }
    fn get_mut(&mut self) -> &mut [u16] {
        assert!(
            self.mutability == Mutability::Mutable,
            "backing storage must be mutable in order to get internal storage mutably"
        );
        unsafe { &mut *(self.s as *mut _) }
    }
}

/// The map background is the method of drawing game maps to the screen. It
/// automatically handles copying the correct portion of a provided map to the
/// assigned block depending on given coordinates.
#[allow(dead_code)]
pub struct BackgroundRegular<'a> {
    register: BackgroundRegister,
    commited_position: Vector2D<i32>,
    shadowed_position: Vector2D<i32>,
    poisoned: bool,
    copy_size: Vector2D<u16>,
    map: Option<Map<'a>>,
}

pub struct Map<'a> {
    store: MapStorage<'a>,
    pub dimensions: Vector2D<u32>,
    pub default: u16,
}

impl<'a> Map<'a> {
    pub fn new(map: &[u16], dimensions: Vector2D<u32>, default: u16) -> Map {
        Map {
            store: MapStorage::new(map),
            dimensions,
            default,
        }
    }
    pub fn new_mutable(map: &mut [u16], dimensions: Vector2D<u32>, default: u16) -> Map {
        Map {
            store: MapStorage::new_mutable(map),
            dimensions,
            default,
        }
    }
    fn get_position(&self, x: i32, y: i32) -> u16 {
        if x < 0 || x as u32 >= self.dimensions.x || y < 0 || y as u32 >= self.dimensions.y {
            self.default
        } else {
            self.store[y as usize * self.dimensions.x as usize + x as usize]
        }
    }
    pub fn get_store(&self) -> &[u16] {
        self.store.get()
    }
    pub fn get_mutable_store(&mut self) -> &mut [u16] {
        self.store.get_mut()
    }
}

pub struct BackgroundRegister {
    background: u8,
    block: u8,
    background_size: BackgroundSize,
    shadowed_register: u16,
}

impl<'a> BackgroundRegister {
    unsafe fn new(background: u8, block: u8, background_size: BackgroundSize) -> Self {
        let mut b = Self {
            background,
            block,
            background_size,
            shadowed_register: 0,
        };
        b.set_block(block);
        b.set_colour_mode(ColourMode::FourBitPerPixel);
        b.set_background_size(background_size);
        b.write_register();
        b
    }

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

    pub fn set_priority(&mut self, p: Priority) {
        unsafe { self.set_shadowed_register_bits(p as u16, 0x2, 0x0) };
    }

    unsafe fn set_shadowed_register_bits(&mut self, value: u16, length: u16, shift: u16) {
        let mask = !(((1 << length) - 1) << shift);
        let new = (self.shadowed_register & mask) | (value << shift);
        self.shadowed_register = new;
    }

    pub fn write_register(&self) {
        unsafe { self.get_register().set(self.shadowed_register) };
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

    unsafe fn set_background_size(&mut self, size: BackgroundSize) {
        self.set_shadowed_register_bits(size as u16, 0x2, 0xE);
    }

    unsafe fn set_position_x_register(&self, x: u16) {
        *((0x0400_0010 + 4 * self.background as usize) as *mut u16) = x
    }
    unsafe fn set_position_y_register(&self, y: u16) {
        *((0x0400_0012 + 4 * self.background as usize) as *mut u16) = y
    }

    pub fn set_position(&self, position: Vector2D<i32>) {
        unsafe {
            self.set_position_x_register((position.x % (32 * 8)) as u16);
            self.set_position_y_register((position.y % (32 * 8)) as u16);
        }
    }

    pub fn get_block(&mut self) -> &mut [[u16; 32]; 32] {
        unsafe { &mut (*MAP)[self.block as usize] }
    }

    pub fn clear_partial(&'a mut self, tile: u16) -> impl Iterator<Item = ()> + 'a {
        self.get_block()
            .iter_mut()
            .flatten()
            .map(move |t| unsafe { (t as *mut u16).write_volatile(tile) })
    }

    pub fn clear(&mut self, tile: u16) {
        self.clear_partial(tile).count();
    }
}

impl<'a, 'b> BackgroundRegular<'a> {
    unsafe fn new(
        background: u8,
        block: u8,
        background_size: BackgroundSize,
    ) -> BackgroundRegular<'a> {
        BackgroundRegular {
            register: BackgroundRegister::new(background, block, background_size),
            commited_position: (0, 0).into(),
            shadowed_position: (0, 0).into(),
            copy_size: (30_u16, 20_u16).into(),
            poisoned: true,
            map: None,
        }
    }

    /// Sets the background to be shown on screen. Requires the background to
    /// have a map enabled otherwise a panic is caused.
    pub fn show(&mut self) {
        assert!(self.map.is_some());
        self.register.show();
    }

    /// Hides the background, nothing from this background is rendered to screen.
    pub fn hide(&mut self) {
        self.register.hide();
    }

    pub fn set_priority(&mut self, p: Priority) {
        self.register.set_priority(p);
    }

    pub fn set_position(&mut self, position: Vector2D<i32>) {
        self.shadowed_position = position;
    }

    pub fn get_map(&mut self) -> Option<&mut Map<'a>> {
        self.poisoned = true;
        self.map.as_mut()
    }

    pub fn set_map(&mut self, map: Map<'a>) {
        self.poisoned = true;
        self.map = Some(map);
    }

    pub fn commit_partial(&'b mut self) -> impl Iterator<Item = ()> + 'b {
        // commit shadowed register
        self.register.write_register();

        let map = self.map.as_ref().unwrap();

        let commited_screen = Rect::new(self.commited_position, self.copy_size.change_base());
        let shadowed_screen = Rect::new(self.shadowed_position, self.copy_size.change_base());

        let iter = if self.poisoned || !shadowed_screen.touches(commited_screen) {
            let positions_to_be_updated = Rect::new(
                self.shadowed_position / 8 - (1, 1).into(),
                self.copy_size.change_base() + (1, 1).into(),
            )
            .iter();

            positions_to_be_updated.chain(Rect::new((0, 0).into(), (0, 0).into()).iter())
        } else {
            let commited_block = self.commited_position / 8;
            let shadowed_block = self.shadowed_position / 8;

            let top_bottom_rect: Rect<i32> = {
                let top_bottom_height = commited_block.y - shadowed_block.y;
                let new_y = if top_bottom_height < 0 {
                    commited_block.y + self.copy_size.y as i32
                } else {
                    shadowed_block.y - 1
                };
                Rect::new(
                    (shadowed_block.x - 1, new_y).into(),
                    (32, top_bottom_height.abs()).into(),
                )
            };

            let left_right_rect: Rect<i32> = {
                let left_right_width = commited_block.x - shadowed_block.x;
                let new_x = if left_right_width < 0 {
                    commited_block.x + self.copy_size.x as i32
                } else {
                    shadowed_block.x - 1
                };
                Rect::new(
                    (new_x, shadowed_block.y - 1).into(),
                    (left_right_width.abs(), 22).into(),
                )
            };

            top_bottom_rect.iter().chain(left_right_rect.iter())
        };

        // update commited position

        self.commited_position = self.shadowed_position;

        self.poisoned = false;

        // update position in registers

        self.register.set_position(self.commited_position);
        let block = self.register.get_block();
        iter.map(move |(x, y)| {
            block[y.rem_euclid(32) as usize][x.rem_euclid(32) as usize] = map.get_position(x, y)
        })
    }

    pub fn commit(&mut self) {
        self.commit_partial().count();
    }
}

fn decide_background_mode(num_regular: u8, num_affine: u8) -> Option<DisplayMode> {
    if num_affine == 0 && num_regular <= 4 {
        Some(DisplayMode::Tiled0)
    } else if num_affine == 1 && num_regular <= 2 {
        Some(DisplayMode::Tiled1)
    } else if num_affine == 2 && num_regular == 0 {
        Some(DisplayMode::Tiled2)
    } else {
        None
    }
}

pub struct BackgroundDistributor {
    used_blocks: u32,
    num_regular: u8,
    num_affine: u8,
}

impl<'b> BackgroundDistributor {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_settings(GraphicsSettings::empty() | GraphicsSettings::SPRITE1_D);
        set_graphics_mode(DisplayMode::Tiled0);
        BackgroundDistributor {
            used_blocks: 0,
            num_regular: 0,
            num_affine: 0,
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
    pub fn get_regular(&mut self) -> Result<BackgroundRegular<'b>, &'static str> {
        let new_mode = decide_background_mode(self.num_regular + 1, self.num_affine)
            .ok_or("there is no mode compatible with the requested backgrounds")?;

        unsafe { set_graphics_mode(new_mode) };

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

        let background = self.num_regular;
        self.num_regular += 1;
        Ok(unsafe { BackgroundRegular::new(background, availiable_block, BackgroundSize::S32x32) })
    }

    pub fn get_raw_regular(&mut self) -> Result<BackgroundRegister, &'static str> {
        let new_mode = decide_background_mode(self.num_regular + 1, self.num_affine)
            .ok_or("there is no mode compatible with the requested backgrounds")?;

        unsafe { set_graphics_mode(new_mode) };

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

        let background = self.num_regular;
        self.num_regular += 1;
        Ok(
            unsafe {
                BackgroundRegister::new(background, availiable_block, BackgroundSize::S32x32)
            },
        )
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
