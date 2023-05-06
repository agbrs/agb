use core::{alloc::Allocator, ptr::NonNull};

use alloc::{
    alloc::Global,
    boxed::Box,
    rc::{Rc, Weak},
    vec::Vec,
};

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd},
    display::palette16::Palette16,
    hash_map::HashMap,
};

use super::{
    sprite::{Size, Sprite},
    BYTES_PER_TILE_4BPP,
};

const PALETTE_SPRITE: usize = 0x0500_0200;
const TILE_SPRITE: usize = 0x06010000;

static SPRITE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || TILE_SPRITE,
        end: || TILE_SPRITE + 1024 * 8 * 4,
    })
};

static PALETTE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || PALETTE_SPRITE,
        end: || PALETTE_SPRITE + 0x200,
    })
};

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
    fn from_static_palette(palette: &'static Palette16) -> PaletteId {
        PaletteId(palette as *const _ as usize)
    }
}

/// This holds loading of static sprites and palettes.
pub struct SpriteLoader {
    static_palette_map: HashMap<PaletteId, Weak<PaletteVramData>>,
    static_sprite_map: HashMap<SpriteId, Weak<SpriteVramData>>,
}

#[derive(Clone, Copy, Debug)]
struct Location(usize);

impl Location {
    fn from_sprite_ptr(d: NonNull<u8>) -> Self {
        Self(((d.as_ptr() as usize) - TILE_SPRITE) / BYTES_PER_TILE_4BPP)
    }
    fn from_palette_ptr(d: NonNull<u8>) -> Self {
        Self((d.as_ptr() as usize - PALETTE_SPRITE) / Palette16::layout().size())
    }
    fn as_palette_ptr(self) -> *mut u8 {
        (self.0 * Palette16::layout().size() + PALETTE_SPRITE) as *mut u8
    }
    fn as_sprite_ptr(self) -> *mut u8 {
        (self.0 * BYTES_PER_TILE_4BPP + TILE_SPRITE) as *mut u8
    }
}

#[derive(Debug)]
struct PaletteVramData {
    location: Location,
}

impl Drop for PaletteVramData {
    fn drop(&mut self) {
        unsafe { PALETTE_ALLOCATOR.dealloc(self.location.as_palette_ptr(), Palette16::layout()) }
    }
}

/// A palette in vram, this is reference counted so it is cheap to Clone.
#[derive(Debug, Clone)]
pub struct PaletteVram {
    data: Rc<PaletteVramData>,
}

impl PaletteVram {
    /// Attempts to allocate a new palette in sprite vram
    pub fn new(palette: &Palette16) -> Result<PaletteVram, LoaderError> {
        let allocated = unsafe { PALETTE_ALLOCATOR.alloc(Palette16::layout()) }
            .ok_or(LoaderError::PaletteFull)?;

        unsafe {
            allocated
                .as_ptr()
                .cast::<u16>()
                .copy_from_nonoverlapping(palette.colours.as_ptr(), palette.colours.len());
        }

        Ok(PaletteVram {
            data: Rc::new(PaletteVramData {
                location: Location::from_palette_ptr(allocated),
            }),
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
        Ok(SpriteVram {
            data: Rc::new(SpriteVramData {
                location: Location::from_sprite_ptr(allocated),
                size,
                palette,
            }),
        })
    }

    pub(crate) fn location(&self) -> u16 {
        self.data.location.0 as u16
    }

    pub(crate) fn size(&self) -> Size {
        self.data.size
    }

    pub(crate) fn palette_location(&self) -> u16 {
        self.data.palette.data.location.0 as u16
    }
}

impl SpriteLoader {
    fn create_sprite_no_insert(
        palette_map: &mut HashMap<PaletteId, Weak<PaletteVramData>>,
        sprite: &'static Sprite,
    ) -> Result<(Weak<SpriteVramData>, SpriteVram), LoaderError> {
        let palette = Self::try_get_vram_palette_asoc(palette_map, sprite.palette)?;

        let sprite = SpriteVram::new(sprite.data, sprite.size, palette)?;
        Ok((Rc::downgrade(&sprite.data), sprite))
    }

    fn try_get_vram_palette_asoc(
        palette_map: &mut HashMap<PaletteId, Weak<PaletteVramData>>,
        palette: &'static Palette16,
    ) -> Result<PaletteVram, LoaderError> {
        let id = PaletteId::from_static_palette(palette);
        Ok(match palette_map.entry(id) {
            crate::hash_map::Entry::Occupied(mut entry) => match entry.get().upgrade() {
                Some(data) => PaletteVram { data },
                None => {
                    let pv = PaletteVram::new(palette)?;
                    entry.insert(Rc::downgrade(&pv.data));
                    pv
                }
            },
            crate::hash_map::Entry::Vacant(entry) => {
                let pv = PaletteVram::new(palette)?;
                entry.insert(Rc::downgrade(&pv.data));
                pv
            }
        })
    }

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
                    let (weak, vram) =
                        Self::create_sprite_no_insert(&mut self.static_palette_map, sprite)?;
                    entry.insert(weak);
                    vram
                }
            },
            crate::hash_map::Entry::Vacant(entry) => {
                let (weak, vram) =
                    Self::create_sprite_no_insert(&mut self.static_palette_map, sprite)?;
                entry.insert(weak);
                vram
            }
        })
    }

    /// Attempts to allocate a static palette
    pub fn try_get_vram_palette(
        &mut self,
        palette: &'static Palette16,
    ) -> Result<PaletteVram, LoaderError> {
        Self::try_get_vram_palette_asoc(&mut self.static_palette_map, palette)
    }

    /// Allocates a sprite to vram, panics if it cannot fit.
    pub fn get_vram_sprite(&mut self, sprite: &'static Sprite) -> SpriteVram {
        self.try_get_vram_sprite(sprite)
            .expect("cannot create sprite")
    }

    /// Allocates a palette to vram, panics if it cannot fit.
    pub fn get_vram_palette(&mut self, palette: &'static Palette16) -> PaletteVram {
        self.try_get_vram_palette(palette)
            .expect("cannot create sprite")
    }

    pub(crate) fn new() -> Self {
        Self {
            static_palette_map: HashMap::new(),
            static_sprite_map: HashMap::new(),
        }
    }

    /// Remove internal references to sprites that no longer exist in vram. If
    /// you neglect calling this, memory will leak over time in relation to the
    /// total number of different sprites used. It will not leak vram.
    pub fn garbage_collect(&mut self) {
        self.static_sprite_map
            .retain(|_, v| Weak::strong_count(v) != 0);
        self.static_palette_map
            .retain(|_, v| Weak::strong_count(v) != 0);
    }
}

impl Default for SpriteLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Sprite data that can be used to create sprites in vram.
pub struct DynamicSprite<A: Allocator = Global> {
    data: Box<[u8], A>,
    size: Size,
}

impl DynamicSprite {
    #[must_use]
    /// Creates a new dynamic sprite.
    pub fn new(size: Size) -> Self {
        Self::new_in(size, Global)
    }
}

impl<A: Allocator> DynamicSprite<A> {
    #[must_use]
    /// Creates a new dynamic sprite of a given size in a given allocator.
    pub fn new_in(size: Size, allocator: A) -> Self {
        let num_bytes = size.number_of_tiles() * BYTES_PER_TILE_4BPP;
        let mut data = Vec::with_capacity_in(num_bytes, allocator);

        data.resize(num_bytes, 0);

        let data = data.into_boxed_slice();

        DynamicSprite { data, size }
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

        let byte_to_modify_in_tile = x / 2 + y * 4;
        let byte_to_modify = tile_number_to_modify * BYTES_PER_TILE_4BPP + byte_to_modify_in_tile;
        let mut byte = self.data[byte_to_modify];
        let parity = (x & 0b1) * 4;

        byte = (byte & !(0b1111 << parity)) | ((paletted_pixel as u8) << parity);
        self.data[byte_to_modify] = byte;
    }

    /// Tries to copy the sprite to vram to be used to set object sprites.
    pub fn try_vram(&self, palette: PaletteVram) -> Result<SpriteVram, LoaderError> {
        SpriteVram::new(&self.data, self.size, palette)
    }

    #[must_use]
    /// Tries to copy the sprite to vram to be used to set object sprites.
    /// Panics if it cannot be allocated.
    pub fn to_vram(&self, palette: PaletteVram) -> SpriteVram {
        self.try_vram(palette).expect("cannot create sprite")
    }
}
