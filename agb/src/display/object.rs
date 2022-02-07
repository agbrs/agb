use core::cell::RefCell;

use hashbrown::{hash_map::Entry, HashMap};

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

const OBJECT_ATTRIBUTE_MEMORY: MemoryMapped1DArray<u16, 512> =
    unsafe { MemoryMapped1DArray::new(0x0700_0000) };
const PALETTE_SPRITE: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0200) };
const TILE_SPRITE: MemoryMapped1DArray<u32, { 1024 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06010000) };

pub struct Sprite {
    palette: &'static Palette16,
    data: &'static [u8],
}

struct SpriteBorrow<'a> {
    id: SpriteId,
    controller: &'a RefCell<SpriteControllerInner>,
}

struct Storage {
    location: u16,
    count: u16,
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

/// The palette id is a thin wrapper around the pointer to the palette in rom
/// and is therefore a unique reference to a palette
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PaletteId(usize);

impl Sprite {
    fn get_id(&'static self) -> SpriteId {
        SpriteId(self as *const _ as usize)
    }
}

impl SpriteController {
    fn get_sprite(&self, sprite: &'static Sprite) -> Option<SpriteBorrow> {
        let inner = self.inner.borrow_mut();
        let id = sprite.get_id();
        if let Some(storage) = inner.sprite.get_mut(&id) {
            storage.count += 1;
            Some(SpriteBorrow {
                id,
                controller: &self.inner,
            })
        } else {
            // allocate a new sprite
            todo!();
        }
    }
}

impl<'a> Drop for SpriteBorrow<'a> {
    fn drop(&mut self) {
        let inner = self.controller.borrow_mut();
        let entry = inner
            .sprite
            .entry(self.id)
            .and_replace_entry_with(|_, mut storage| {
                storage.count -= 1;
                if storage.count == 0 {
                    None
                } else {
                    Some(storage)
                }
            });

        match entry {
            Entry::Vacant(_) => {
                // free the underlying resource.
                // palette might be unused too.
            }
            _ => {}
        }
    }
}

impl ObjectController {}
