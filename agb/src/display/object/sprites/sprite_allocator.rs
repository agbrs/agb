use core::{alloc::Allocator, cell::Cell, hint::assert_unchecked, ptr::NonNull};

use alloc::{
    boxed::Box,
    rc::{Rc, Weak},
};

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd, impl_zst_allocator},
    display::palette16::Palette16,
    hash_map::HashMap,
};

use super::{
    sprite::{MultiPalette, Palette, Size, Sprite},
    BYTES_PER_TILE_4BPP,
};

pub const PALETTE_SPRITE: usize = 0x0500_0200;
pub const TILE_SPRITE: usize = 0x06010000;

static SPRITE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || TILE_SPRITE,
        end: || TILE_SPRITE + 1024 * 8 * 4,
    })
};

pub struct SpriteAllocator;

impl_zst_allocator!(SpriteAllocator, SPRITE_ALLOCATOR);

struct PaletteAllocator {
    allocation: Cell<u16>,
}
#[derive(Debug)]
struct MultiPaletteAllocation(u16);

#[derive(Debug)]
struct SinglePaletteAllocation(u8);

impl Drop for SinglePaletteAllocation {
    fn drop(&mut self) {
        PALETTE_ALLOCATOR.deallocate_single(self);
    }
}

impl Drop for MultiPaletteAllocation {
    fn drop(&mut self) {
        PALETTE_ALLOCATOR.deallocate_multi(self);
    }
}

const PALETTE_VRAM: *mut [Palette16; 16] = PALETTE_SPRITE as *mut _;

impl PaletteAllocator {
    const fn new() -> Self {
        Self {
            allocation: Cell::new(0),
        }
    }

    /// For allocating a multi palette
    fn allocate_multiple(&self, palette: &MultiPalette) -> Option<MultiPaletteAllocation> {
        unsafe {
            assert_unchecked(palette.palettes().len() <= 16);
            assert_unchecked(!palette.palettes().is_empty());
            assert_unchecked(16 - palette.palettes().len() > palette.first_index() as usize);
        }

        let claim = (1u32 << palette.palettes().len()) - 1;
        let claim = claim << palette.first_index();
        unsafe {
            assert_unchecked(claim <= u16::MAX as u32);
        }
        let claim = claim as u16;
        let currently_allocated = self.allocation.get();
        if currently_allocated & claim != 0 {
            return None;
        }

        self.allocation.set(currently_allocated | claim);

        // copy the data across
        unsafe {
            let p = (&mut (*PALETTE_VRAM)[palette.first_index() as usize]) as *mut Palette16;
            p.copy_from_nonoverlapping(palette.palettes().as_ptr(), palette.palettes().len());
        }

        Some(MultiPaletteAllocation(claim))
    }

    fn allocate_single(&self, palette: &Palette16) -> Option<SinglePaletteAllocation> {
        let currently_allocated = self.allocation.get();

        for idx in 0..16 {
            let claim = 1u16 << idx;

            if currently_allocated & claim == 0 {
                self.allocation.set(currently_allocated | claim);
                unsafe {
                    let palette_to_write_to = &mut (*PALETTE_VRAM)[idx] as *mut Palette16;
                    palette_to_write_to.copy_from_nonoverlapping(palette, 1);
                }
                return Some(SinglePaletteAllocation(idx as u8));
            }
        }

        None
    }

    fn deallocate_single(&self, claim: &SinglePaletteAllocation) {
        assert!(claim.0 < 16);

        let allocation = self.allocation.get();

        self.allocation.set(allocation & !(1 << claim.0));
    }

    fn deallocate_multi(&self, claim: &MultiPaletteAllocation) {
        let allocation = self.allocation.get();

        self.allocation.set(allocation & !(claim.0));
    }
}

/// Not (yet) multi threaded
unsafe impl Sync for PaletteAllocator {}

static PALETTE_ALLOCATOR: PaletteAllocator = PaletteAllocator::new();

/// The Sprite Id is a thin wrapper around the pointer to the sprite in
/// rom and is therefore a unique identifier to a sprite
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SpriteId(usize);

impl SpriteId {
    fn from_static_sprite(sprite: &'static Sprite) -> SpriteId {
        SpriteId(sprite as *const _ as usize)
    }
}

/// The palette id is a thin wrapper around the pointer to the palette in rom
/// and is therefore a unique reference to a palette
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PaletteId(usize);

impl PaletteId {
    fn new(palette: &'static Palette16) -> Self {
        Self(palette as *const _ as usize)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct MultiPaletteId(usize);

impl MultiPaletteId {
    fn new(palette: &'static MultiPalette) -> Self {
        Self(palette as *const _ as usize)
    }
}

struct PaletteLoader {
    static_palette_map: HashMap<PaletteId, Weak<SinglePaletteAllocation>>,
    static_multi_palette_map: HashMap<MultiPaletteId, Weak<MultiPaletteAllocation>>,
}

/// This holds loading of static sprites and palettes.
pub struct SpriteLoader {
    palettes: PaletteLoader,
    static_sprite_map: HashMap<SpriteId, Weak<SpriteVramData>>,
}

#[derive(Clone, Copy, Debug)]
struct Location(usize);

impl Location {
    fn from_sprite_ptr(d: NonNull<u8>) -> Self {
        Self(((d.as_ptr() as usize) - TILE_SPRITE) / BYTES_PER_TILE_4BPP)
    }
    fn as_sprite_ptr(self) -> *mut u8 {
        (self.0 * BYTES_PER_TILE_4BPP + TILE_SPRITE) as *mut u8
    }
}

#[derive(Debug, Clone)]
/// A palette in vram, this is reference counted so it is cheap to Clone.
pub enum PaletteVram {
    /// A single palette designed to be used in 4 bit per pixel mode
    Single(SinglePaletteVram),
    /// Multiple palettes designed to be used in 8 bit per pixel / 256 colour mode
    Multi(MultiPaletteVram),
}

impl PaletteVram {
    /// Creates an instance of a single palette in vram
    pub fn new(palette: &Palette16) -> Result<Self, LoaderError> {
        Ok(PaletteVram::Single(SinglePaletteVram::new(palette)?))
    }

    /// Creates an instance of a single palette in vram
    pub fn new_multi(palette: &MultiPalette) -> Result<Self, LoaderError> {
        Ok(PaletteVram::Multi(MultiPaletteVram::new(palette)?))
    }
}

/// A palette in vram, this is reference counted so it is cheap to Clone.
#[derive(Debug, Clone)]
pub struct SinglePaletteVram {
    data: Rc<SinglePaletteAllocation>,
}

impl SinglePaletteVram {
    pub fn new(palette: &Palette16) -> Result<Self, LoaderError> {
        Ok(Self {
            data: Rc::new(
                PALETTE_ALLOCATOR
                    .allocate_single(palette)
                    .ok_or(LoaderError::PaletteFull)?,
            ),
        })
    }
}

#[derive(Debug, Clone)]
pub struct MultiPaletteVram {
    data: Rc<MultiPaletteAllocation>,
}

impl MultiPaletteVram {
    pub fn new(palette: &MultiPalette) -> Result<Self, LoaderError> {
        Ok(Self {
            data: Rc::new(
                PALETTE_ALLOCATOR
                    .allocate_multiple(palette)
                    .ok_or(LoaderError::PaletteFull)?,
            ),
        })
    }
}

#[derive(Debug)]
struct SpriteVramData {
    location: Location,
    size: Size,
    palette: PaletteVram,
}

impl Drop for SpriteVramData {
    fn drop(&mut self) {
        unsafe { SPRITE_ALLOCATOR.dealloc(self.location.as_sprite_ptr(), self.size.layout()) }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoaderError {
    SpriteFull,
    PaletteFull,
}

/// A sprite that is currently loaded into vram.
///
/// This is referenced counted such that clones of this are cheap and can be
/// reused between objects. When nothing references the sprite it gets
/// deallocated from vram.
///
/// You can create one of these either via the [DynamicSprite] interface, which
/// allows you to generate sprites at run time, or via a [SpriteLoader] (or
/// [OamManaged][super::super::OamManaged]).
#[derive(Clone, Debug)]
pub struct SpriteVram {
    data: Rc<SpriteVramData>,
}

impl SpriteVram {
    fn new(data: &[u8], size: Size, palette: PaletteVram) -> Result<SpriteVram, LoaderError> {
        let allocated =
            unsafe { SPRITE_ALLOCATOR.alloc(size.layout()) }.ok_or(LoaderError::SpriteFull)?;
        unsafe {
            allocated
                .as_ptr()
                .copy_from_nonoverlapping(data.as_ptr(), data.len());
        }
        Ok(unsafe { Self::from_location_size(allocated, size, palette) })
    }

    unsafe fn from_location_size(
        data: NonNull<u8>,
        size: Size,
        palette: PaletteVram,
    ) -> SpriteVram {
        SpriteVram {
            data: Rc::new(SpriteVramData {
                location: Location::from_sprite_ptr(data),
                size,
                palette,
            }),
        }
    }
    pub(crate) fn palette_single(&self) -> Option<u8> {
        match &self.data.palette {
            PaletteVram::Single(single_palette_vram) => Some(single_palette_vram.data.0),
            PaletteVram::Multi(_) => None,
        }
    }

    pub(crate) fn location(&self) -> u16 {
        self.data.location.0 as u16
    }

    pub(crate) fn size(&self) -> Size {
        self.data.size
    }
}

impl PaletteLoader {
    fn try_get_vram_palette(&mut self, palette: Palette) -> Result<PaletteVram, LoaderError> {
        Ok(match palette {
            Palette::Single(palette) => {
                PaletteVram::Single(self.try_get_vram_palette_single(palette)?)
            }
            Palette::Multi(palette) => {
                PaletteVram::Multi(self.try_get_vram_palette_multi(palette)?)
            }
        })
    }

    fn try_get_vram_palette_single(
        &mut self,
        palette: &'static Palette16,
    ) -> Result<SinglePaletteVram, LoaderError> {
        let id = PaletteId::new(palette);
        Ok(match self.static_palette_map.entry(id) {
            crate::hash_map::Entry::Occupied(mut entry) => match entry.get().upgrade() {
                Some(data) => SinglePaletteVram { data },
                None => {
                    let pv = SinglePaletteVram::new(palette)?;
                    entry.insert(Rc::downgrade(&pv.data));
                    pv
                }
            },
            crate::hash_map::Entry::Vacant(entry) => {
                let pv = SinglePaletteVram::new(palette)?;
                entry.insert(Rc::downgrade(&pv.data));
                pv
            }
        })
    }

    fn try_get_vram_palette_multi(
        &mut self,
        palette: &'static MultiPalette,
    ) -> Result<MultiPaletteVram, LoaderError> {
        let id = MultiPaletteId::new(palette);
        Ok(match self.static_multi_palette_map.entry(id) {
            crate::hash_map::Entry::Occupied(mut entry) => match entry.get().upgrade() {
                Some(data) => MultiPaletteVram { data },
                None => {
                    let pv = MultiPaletteVram::new(palette)?;
                    entry.insert(Rc::downgrade(&pv.data));
                    pv
                }
            },
            crate::hash_map::Entry::Vacant(entry) => {
                let pv = MultiPaletteVram::new(palette)?;
                entry.insert(Rc::downgrade(&pv.data));
                pv
            }
        })
    }

    fn create_sprite_no_insert(
        &mut self,
        sprite: &'static Sprite,
    ) -> Result<(Weak<SpriteVramData>, SpriteVram), LoaderError> {
        let palette = self.try_get_vram_palette(sprite.palette)?;

        let sprite = SpriteVram::new(sprite.data, sprite.size, palette)?;
        Ok((Rc::downgrade(&sprite.data), sprite))
    }

    fn garbage_collect(&mut self) {
        self.static_palette_map
            .retain(|_, v| Weak::strong_count(v) != 0);
        self.static_multi_palette_map
            .retain(|_, v| Weak::strong_count(v) != 0);
    }
}

impl SpriteLoader {
    /// Attempts to get a sprite
    pub fn try_get_vram_sprite(
        &mut self,
        sprite: &'static Sprite,
    ) -> Result<SpriteVram, LoaderError> {
        // check if we already have the sprite in vram

        let id = SpriteId::from_static_sprite(sprite);

        Ok(match self.static_sprite_map.entry(id) {
            crate::hash_map::Entry::Occupied(mut entry) => match entry.get().upgrade() {
                Some(data) => SpriteVram { data },
                None => {
                    let (weak, vram) = self.palettes.create_sprite_no_insert(sprite)?;
                    entry.insert(weak);
                    vram
                }
            },
            crate::hash_map::Entry::Vacant(entry) => {
                let (weak, vram) = self.palettes.create_sprite_no_insert(sprite)?;
                entry.insert(weak);
                vram
            }
        })
    }

    /// Attempts to allocate a static palette
    pub fn try_get_vram_palette(&mut self, palette: Palette) -> Result<PaletteVram, LoaderError> {
        self.palettes.try_get_vram_palette(palette)
    }

    /// Allocates a sprite to vram, panics if it cannot fit.
    pub fn get_vram_sprite(&mut self, sprite: &'static Sprite) -> SpriteVram {
        self.try_get_vram_sprite(sprite)
            .expect("cannot create sprite")
    }

    /// Allocates a palette to vram, panics if it cannot fit.
    pub fn get_vram_palette(&mut self, palette: Palette) -> PaletteVram {
        self.try_get_vram_palette(palette)
            .expect("cannot create sprite")
    }

    pub(crate) fn new() -> Self {
        Self {
            palettes: PaletteLoader {
                static_palette_map: HashMap::new(),
                static_multi_palette_map: HashMap::new(),
            },
            static_sprite_map: HashMap::new(),
        }
    }

    /// Remove internal references to sprites that no longer exist in vram. If
    /// you neglect calling this, memory will leak over time in relation to the
    /// total number of different sprites used. It will not leak vram.
    pub fn garbage_collect(&mut self) {
        self.static_sprite_map
            .retain(|_, v| Weak::strong_count(v) != 0);
        self.palettes.garbage_collect();
    }
}

impl Default for SpriteLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Sprite data that can be used to create sprites in vram.
pub struct DynamicSprite {
    data: Box<[u16], SpriteAllocator>,
    size: Size,
}

impl Clone for DynamicSprite {
    fn clone(&self) -> Self {
        let allocation = SpriteAllocator
            .allocate(self.size.layout())
            .expect("cannot allocate dynamic sprite");

        let allocation = core::ptr::slice_from_raw_parts_mut(
            allocation.as_ptr() as *mut _,
            allocation.len() / 2,
        );

        let mut data = unsafe { Box::from_raw_in(allocation, SpriteAllocator) };

        data.clone_from_slice(&self.data);

        Self {
            data,
            size: self.size,
        }
    }
}

impl DynamicSprite {
    /// Creates a new dynamic sprite of a given size
    pub fn try_new(size: Size) -> Result<Self, LoaderError> {
        let allocation = SpriteAllocator
            .allocate_zeroed(size.layout())
            .map_err(|_| LoaderError::SpriteFull)?;

        let allocation = core::ptr::slice_from_raw_parts_mut(
            allocation.as_ptr() as *mut _,
            allocation.len() / 2,
        );

        let data = unsafe { Box::from_raw_in(allocation, SpriteAllocator) };

        Ok(DynamicSprite { data, size })
    }

    #[must_use]
    /// Creates a new dynamic sprite of a given size
    pub fn new(size: Size) -> Self {
        Self::try_new(size).expect("couldn't allocate dynamic sprite")
    }

    /// Set the pixel of a sprite to a given paletted pixel. Panics if the
    /// coordinate is out of range of the sprite or if the paletted pixel is
    /// greater than 4 bits.
    pub fn set_pixel(&mut self, x: usize, y: usize, paletted_pixel: usize) {
        assert!(paletted_pixel < 0x10);

        let (sprite_pixel_x, sprite_pixel_y) = self.size.to_width_height();
        assert!(x < sprite_pixel_x, "x too big for sprite size");
        assert!(y < sprite_pixel_y, "y too big for sprite size");

        let (sprite_tile_x, _) = self.size.to_tiles_width_height();

        let (adjust_tile_x, adjust_tile_y) = (x / 8, y / 8);

        let tile_number_to_modify = adjust_tile_x + adjust_tile_y * sprite_tile_x;

        let (x_in_tile, y_in_tile) = (x % 8, y % 8);

        let half_word_to_modify_in_tile = x_in_tile / 4 + y_in_tile * 2;

        let half_word_to_modify =
            tile_number_to_modify * BYTES_PER_TILE_4BPP / 2 + half_word_to_modify_in_tile;
        let mut half_word = self.data[half_word_to_modify];

        let nibble_to_modify = (x % 4) * 4;

        half_word = (half_word & !(0b1111 << nibble_to_modify))
            | ((paletted_pixel as u16) << nibble_to_modify);
        self.data[half_word_to_modify] = half_word;
    }

    /// Wipes the sprite
    pub fn clear(&mut self, paletted_pixel: usize) {
        assert!(paletted_pixel < 0x10);
        let reset = (paletted_pixel
            | (paletted_pixel << 4)
            | (paletted_pixel << 8)
            | (paletted_pixel << 12)) as u16;
        self.data.fill(reset);
    }

    #[must_use]
    /// Tries to copy the sprite to vram to be used to set object sprites.
    /// Panics if it cannot be allocated.
    pub fn to_vram(self, palette: PaletteVram) -> SpriteVram {
        let data = unsafe { NonNull::new_unchecked(Box::leak(self.data).as_mut_ptr()) };

        unsafe { SpriteVram::from_location_size(data.cast(), self.size, palette) }
    }
}
