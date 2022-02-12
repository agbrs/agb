use core::alloc::Layout;
use core::cell::RefCell;
use core::ptr::NonNull;

use hashbrown::{hash_map::Entry, HashMap};

const BYTES_PER_TILE_4BPP: usize = 32;

use super::palette16::Palette16;
use super::{palette16, Priority, DISPLAY_CONTROL};
use crate::agb_alloc::block_allocator::BlockAllocator;
use crate::agb_alloc::bump_allocator::StartEnd;
use crate::bitarray::Bitarray;
use crate::fixnum::Vector2D;
use crate::memory_mapped::MemoryMapped1DArray;

static SPRITE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || 0x06010000,
        end: || 0x06010000 + 1024 * 8 * 4,
    })
};

static PALETTE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || 0x0500_0200,
        end: || 0x0500_0400,
    })
};

const OBJECT_ATTRIBUTE_MEMORY: MemoryMapped1DArray<u16, 512> =
    unsafe { MemoryMapped1DArray::new(0x0700_0000) };
const PALETTE_SPRITE: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0200) };
const TILE_SPRITE: MemoryMapped1DArray<u32, { 1024 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06010000) };

pub struct Sprite {
    palette: &'static Palette16,
    data: &'static [u8],
    size: Size,
}

#[derive(Clone, Copy)]
pub enum Size {
    // stored as attr0 attr1
    S8x8 = 0b00_00,
    S16x16 = 0b00_01,
    S32x32 = 0b00_10,
    S64x64 = 0b00_11,

    S16x8 = 0b01_00,
    S32x8 = 0b01_01,
    S32x16 = 0b01_10,
    S64x32 = 0b01_11,

    S8x16 = 0b10_00,
    S8x32 = 0b10_01,
    S16x32 = 0b10_10,
    S32x64 = 0b10_11,
}

impl Size {
    fn number_of_tiles(self) -> usize {
        match self {
            S8x8 => 1,
            S16x16 => 4,
            S32x32 => 16,
            S64x64 => 64,
            S16x8 => 2,
            S32x8 => 4,
            S32x16 => 8,
            S64x32 => 32,
            S8x16 => 2,
            S8x32 => 4,
            S16x32 => 8,
            S32x64 => 32,
        }
    }
}

struct SpriteBorrow<'a> {
    id: SpriteId,
    sprite_location: u16,
    palette_location: u16,
    controller: &'a RefCell<SpriteControllerInner>,
}

struct Storage {
    location: u16,
    count: u16,
}

impl Storage {
    fn from_sprite_ptr(d: NonNull<u8>) -> Self {
        Self {
            location: (((d.as_ptr() as usize) - 0x06010000) / BYTES_PER_TILE_4BPP) as u16,
            count: 1,
        }
    }
    fn from_palette_ptr(d: NonNull<u8>) -> Self {
        Self {
            location: ((d.as_ptr() as usize - 0x0500_0200) / Palette16::layout().size()) as u16,
            count: 1,
        }
    }
    fn to_palette_ptr(&self) -> *mut u8 {
        (self.location as usize * Palette16::layout().size() + 0x0500_0200) as *mut u8
    }
}

pub struct Object<'a> {
    sprite: SpriteBorrow<'a>,
}

struct SpriteControllerInner {
    palette: HashMap<PaletteId, Storage>,
    sprite: HashMap<SpriteId, Storage>,
}

pub struct SpriteController {
    inner: RefCell<SpriteControllerInner>,
}

pub struct ObjectController {}

/// The Sprite Id is a thin wrapper around the pointer to the sprite in
/// rom and is therefore a unique identifier to a sprite
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SpriteId(usize);

impl SpriteId {
    fn get_sprite(self) -> &'static Sprite {
        // # Safety
        // This must be constructed using the get_id of a sprite, so
        // they are always valid and always static
        unsafe { (self.0 as *const Sprite).as_ref().unwrap_unchecked() }
    }
}

/// The palette id is a thin wrapper around the pointer to the palette in rom
/// and is therefore a unique reference to a palette
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PaletteId(usize);

impl PaletteId {
    fn get_palette(self) -> &'static Palette16 {
        unsafe { (self.0 as *const Palette16).as_ref().unwrap_unchecked() }
    }
}

impl Palette16 {
    fn get_id(&'static self) -> PaletteId {
        PaletteId(self as *const _ as usize)
    }
    const fn layout() -> Layout {
        Layout::new::<Self>()
    }
}

impl Sprite {
    fn get_id(&'static self) -> SpriteId {
        SpriteId(self as *const _ as usize)
    }
    const fn layout(&self) -> Layout {
        Layout::from_size_align(self.size.number_of_tiles() * BYTES_PER_TILE_4BPP, 8).unwrap()
    }
}

impl SpriteController {
    fn get_sprite(&self, sprite: &'static Sprite) -> Option<SpriteBorrow> {
        let inner = self.inner.borrow_mut();
        let id = sprite.get_id();
        if let Some(storage) = inner.sprite.get_mut(&id) {
            storage.count += 1;
            let palette_location = inner.get_palette(sprite.palette).unwrap();
            Some(SpriteBorrow {
                id,
                palette_location,
                sprite_location: storage.location,
                controller: &self.inner,
            })
        } else {
            // layout is non zero sized, so this is safe to call

            let dest = unsafe { SPRITE_ALLOCATOR.alloc(sprite.layout())? };
            let palette_location = inner.get_palette(sprite.palette);
            let palette_location = match palette_location {
                Some(a) => a,
                None => {
                    unsafe { SPRITE_ALLOCATOR.dealloc(dest.as_ptr(), sprite.layout()) }
                    return None;
                }
            };

            let storage = Storage::from_sprite_ptr(dest);
            inner.sprite.insert(id, storage);

            Some(SpriteBorrow {
                id,
                controller: &self.inner,
                palette_location,
                sprite_location: storage.location,
            })
        }
    }
}

impl SpriteControllerInner {
    fn get_palette(&mut self, palette: &'static Palette16) -> Option<u16> {
        let id = palette.get_id();
        if let Some(storage) = self.palette.get_mut(&id) {
            storage.count += 1;
            Some(storage.location)
        } else {
            let dest = unsafe { PALETTE_ALLOCATOR.alloc(Palette16::layout())? };
            let storage = Storage::from_palette_ptr(dest);
            self.palette.insert(id, storage);

            Some(storage.location)
        }
    }

    fn return_palette(&mut self, palette: &'static Palette16) {
        let id = palette.get_id();
        self.palette
            .entry(id)
            .and_replace_entry_with(|_, mut storage| {
                storage.count -= 1;
                if storage.count == 0 {
                    unsafe {
                        PALETTE_ALLOCATOR.dealloc(storage.to_palette_ptr(), Palette16::layout());
                    }
                    None
                } else {
                    Some(storage)
                }
            });
    }
}

impl<'a> Drop for SpriteBorrow<'a> {
    fn drop(&mut self) {
        let inner = self.controller.borrow_mut();
        inner
            .sprite
            .entry(self.id)
            .and_replace_entry_with(|_, mut storage| {
                storage.count -= 1;
                if storage.count == 0 {
                    inner.return_palette(self.id.get_sprite().palette);
                    None
                } else {
                    Some(storage)
                }
            });
    }
}

impl ObjectController {}
