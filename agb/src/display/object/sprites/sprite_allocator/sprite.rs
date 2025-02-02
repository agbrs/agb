use core::{alloc::Allocator, ptr::NonNull};

use alloc::rc::Rc;

use crate::{
    agb_alloc::{
        block_allocator::BlockAllocator, bump_allocator::StartEnd, impl_zst_allocator,
        single_allocator::create_allocator_arena,
    },
    display::object::{sprites::BYTES_PER_TILE_4BPP, Size, Sprite},
    ExternalAllocator,
};

use super::{palette::PaletteVram, LoaderError};

pub const TILE_SPRITE: usize = 0x06010000;

static SPRITE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || TILE_SPRITE,
        end: || TILE_SPRITE + 1024 * 8 * 4,
    })
};

pub struct SpriteAllocator;

impl_zst_allocator!(SpriteAllocator, SPRITE_ALLOCATOR);

struct RcInner<T> {
    _counts: [usize; 2],
    _t: T,
}

create_allocator_arena!(SpriteArena, ExternalAllocator, RcInner<SpriteVramData>);

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
    sprite: SpriteVramInner,
    palette: PaletteVram,
}

impl SpriteVram {
    #[must_use]
    pub fn new(sprite: SpriteVramInner, palette: PaletteVram) -> Self {
        Self { sprite, palette }
    }

    #[must_use]
    pub fn location(&self) -> SpriteLocation {
        self.sprite.0.sprite_index
    }

    #[must_use]
    pub fn size(&self) -> Size {
        self.sprite.0.size
    }

    #[must_use]
    pub fn single_palette_index(&self) -> Option<u8> {
        self.palette.single_palette_index()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteLocation(u16);

impl SpriteLocation {
    pub fn from_ptr(ptr: NonNull<u8>) -> Self {
        Self((((ptr.as_ptr() as usize) - TILE_SPRITE) / BYTES_PER_TILE_4BPP) as u16)
    }

    pub fn to_ptr(self) -> NonNull<u8> {
        unsafe {
            NonNull::new_unchecked((self.0 as usize * BYTES_PER_TILE_4BPP + TILE_SPRITE) as *mut u8)
        }
    }

    pub(crate) fn idx(self) -> u16 {
        self.0
    }
}

#[derive(Debug)]
#[repr(align(4))]
struct SpriteVramData {
    sprite_index: SpriteLocation,
    size: Size,
    multi_palette: bool,
}

#[derive(Clone, Debug)]
pub struct SpriteVramInner(Rc<SpriteVramData, SpriteArena>);

impl SpriteVramInner {
    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.0)
    }

    pub fn new(data: &[u8], size: Size, multi: bool) -> Result<SpriteVramInner, LoaderError> {
        let allocated =
            unsafe { SPRITE_ALLOCATOR.alloc(size.layout(multi)) }.ok_or(LoaderError::SpriteFull)?;
        unsafe {
            allocated
                .as_ptr()
                .copy_from_nonoverlapping(data.as_ptr(), data.len());
        }

        Ok(SpriteVramInner(Rc::new_in(
            SpriteVramData {
                sprite_index: SpriteLocation::from_ptr(allocated),
                multi_palette: multi,
                size,
            },
            SpriteArena,
        )))
    }

    pub fn new_from_sprite(sprite: &Sprite) -> Result<SpriteVramInner, LoaderError> {
        Self::new(sprite.data, sprite.size, sprite.palette.is_multi())
    }

    pub unsafe fn new_from_allocated(
        sprite_index: SpriteLocation,
        size: Size,
        multi_palette: bool,
    ) -> Self {
        SpriteVramInner(Rc::new_in(
            SpriteVramData {
                sprite_index,
                size,
                multi_palette,
            },
            SpriteArena,
        ))
    }
}

impl Drop for SpriteVramData {
    fn drop(&mut self) {
        unsafe {
            SPRITE_ALLOCATOR.dealloc(
                self.sprite_index.to_ptr().as_ptr(),
                self.size.layout(self.multi_palette),
            );
        }
    }
}
