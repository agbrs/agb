use core::ptr::NonNull;

use alloc::rc::{Rc, Weak};

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
pub struct StaticSpriteLoader {
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

#[derive(Debug)]
pub struct PaletteVram {
    data: Rc<PaletteVramData>,
}

impl PaletteVram {
    fn new(palette: &Palette16) -> Option<PaletteVram> {
        let allocated = unsafe { PALETTE_ALLOCATOR.alloc(Palette16::layout()) }?;

        unsafe {
            allocated
                .as_ptr()
                .cast::<u16>()
                .copy_from_nonoverlapping(palette.colours.as_ptr(), palette.colours.len());
        }

        Some(PaletteVram {
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

#[derive(Clone, Debug)]
pub struct SpriteVram {
    data: Rc<SpriteVramData>,
}

impl SpriteVram {
    fn new(data: &[u8], size: Size, palette: PaletteVram) -> Option<SpriteVram> {
        let allocated = unsafe { SPRITE_ALLOCATOR.alloc(size.layout()) }?;
        unsafe {
            allocated
                .as_ptr()
                .copy_from_nonoverlapping(data.as_ptr(), data.len());
        }
        Some(SpriteVram {
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

impl StaticSpriteLoader {
    fn create_sprite_no_insert(
        palette_map: &mut HashMap<PaletteId, Weak<PaletteVramData>>,
        sprite: &'static Sprite,
    ) -> Option<(Weak<SpriteVramData>, SpriteVram)> {
        let palette = Self::try_get_vram_palette_asoc(palette_map, sprite.palette)?;

        let sprite = SpriteVram::new(sprite.data, sprite.size, palette)?;
        Some((Rc::downgrade(&sprite.data), sprite))
    }

    fn try_get_vram_palette_asoc(
        palette_map: &mut HashMap<PaletteId, Weak<PaletteVramData>>,
        palette: &'static Palette16,
    ) -> Option<PaletteVram> {
        let id = PaletteId::from_static_palette(palette);
        Some(match palette_map.entry(id) {
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

    pub fn try_get_vram_sprite(&mut self, sprite: &'static Sprite) -> Option<SpriteVram> {
        // check if we already have the sprite in vram

        let id = SpriteId::from_static_sprite(sprite);

        Some(match self.static_sprite_map.entry(id) {
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

    pub fn try_get_vram_palette(&mut self, palette: &'static Palette16) -> Option<PaletteVram> {
        Self::try_get_vram_palette_asoc(&mut self.static_palette_map, palette)
    }

    pub fn get_vram_sprite(&mut self, sprite: &'static Sprite) -> SpriteVram {
        self.try_get_vram_sprite(sprite)
            .expect("no free sprite slots")
    }

    pub fn get_vram_palette(&mut self, palette: &'static Palette16) -> PaletteVram {
        self.try_get_vram_palette(palette)
            .expect("no free palette slots")
    }

    pub(crate) fn new() -> Self {
        Self {
            static_palette_map: HashMap::new(),
            static_sprite_map: HashMap::new(),
        }
    }

    pub fn garbage_collect(&mut self) {
        self.static_sprite_map
            .retain(|_, v| Weak::strong_count(v) != 0);
        self.static_palette_map
            .retain(|_, v| Weak::strong_count(v) != 0);
    }
}

impl Default for StaticSpriteLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Sprite data that can be used to create sprites in vram.
pub struct DynamicSprite<'a> {
    data: &'a [u8],
    size: Size,
}

impl DynamicSprite<'_> {
    #[must_use]
    /// Creates a new dynamic sprite from underlying bytes. Note that despite
    /// being an array of u8, this must be aligned to at least a 2 byte
    /// boundary.
    pub fn new(data: &[u8], size: Size) -> DynamicSprite {
        let ptr = &data[0] as *const _ as usize;
        if ptr % 2 != 0 {
            panic!("data is not aligned to a 2 byte boundary");
        }
        if data.len() != size.number_of_tiles() * BYTES_PER_TILE_4BPP {
            panic!(
                "data is not of expected length, got {} expected {}",
                data.len(),
                size.number_of_tiles() * BYTES_PER_TILE_4BPP
            );
        }
        DynamicSprite { data, size }
    }

    #[must_use]
    /// Tries to copy the sprite to vram to be used to set object sprites.
    /// Returns None if there is no room in sprite vram.
    pub fn try_vram(&self, palette: PaletteVram) -> Option<SpriteVram> {
        SpriteVram::new(self.data, self.size, palette)
    }

    #[must_use]
    /// Tries to copy the sprite to vram to be used to set object sprites.
    /// Panics if there is no room in sprite vram.
    pub fn to_vram(&self, palette: PaletteVram) -> SpriteVram {
        self.try_vram(palette)
            .expect("No slot for sprite available")
    }
}
