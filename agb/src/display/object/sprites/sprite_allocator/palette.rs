use core::{alloc::Allocator, cell::Cell, hint::assert_unchecked, ptr::NonNull};

use crate::{
    ExternalAllocator,
    agb_alloc::single_allocator::create_allocator_arena,
    display::{
        object::{PaletteMulti, sprites::sprite::Palette},
        palette16::Palette16,
    },
    refcount::{RefCount, RefCountInner},
};

use super::{LoaderError, SPRITE_LOADER};

pub const PALETTE_SPRITE: usize = 0x0500_0200;

create_allocator_arena!(
    PaletteArena,
    ExternalAllocator,
    RefCountInner<PaletteAllocation>
);

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
    fn allocate_multiple(&self, palette: &PaletteMulti) -> Option<MultiPaletteAllocation> {
        unsafe {
            assert_unchecked(palette.palettes().len() <= 16);
            assert_unchecked(!palette.palettes().is_empty());
            assert_unchecked(16 - palette.palettes().len() >= palette.first_index() as usize);
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

#[derive(Debug)]
#[repr(align(4))]
#[expect(dead_code, reason = "the drop implementation is used and is important")]
enum PaletteAllocation {
    Single(SinglePaletteAllocation),
    Multi(MultiPaletteAllocation),
}

type RefCountedAllocation = RefCount<PaletteAllocation, PaletteArena>;

/// A palette containing 16 colours that is currently allocated to vram. To use
/// this palette will require 4 bits per pixel.
#[derive(Clone, Debug)]
pub struct PaletteVramSingle(RefCountedAllocation);

impl PaletteVramSingle {
    /// Gets the general PaletteVram that represents this palette. This is
    /// common to both single and multi palettes.
    #[must_use]
    pub fn palette(self) -> PaletteVram {
        PaletteVram(self.0)
    }

    /// Allocates a palette into vram from a palette. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn try_allocate_new(palette: &Palette16) -> Result<Self, LoaderError> {
        let allocation = PALETTE_ALLOCATOR
            .allocate_single(palette)
            .ok_or(LoaderError::PaletteFull)?;
        let allocation = PaletteAllocation::Single(allocation);

        Ok(Self(RefCount::new_in(allocation, PaletteArena)))
    }

    #[must_use]
    /// Allocates the palette sharing an existing allocation where possible
    pub fn new(palette: &'static Palette16) -> Self {
        unsafe {
            SPRITE_LOADER
                .palette(Palette::Single(palette))
                .expect("palette unallocatable")
                .single()
                .unwrap_unchecked()
        }
    }
}

/// A palette that can contain more than 16 colours allocated to vram. To use
/// this palette will require 8 bits per pixel.
#[derive(Clone, Debug)]
pub struct PaletteVramMulti(RefCountedAllocation);

impl PaletteVramMulti {
    /// Gets the general palette that represents this palette. This is common to
    /// both single and multi palettes.
    #[must_use]
    pub fn palette(self) -> PaletteVram {
        PaletteVram(self.0)
    }

    /// Allocates a palette into vram from a palette. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn try_allocate_new(palette: &PaletteMulti) -> Result<Self, LoaderError> {
        let allocation = PALETTE_ALLOCATOR
            .allocate_multiple(palette)
            .ok_or(LoaderError::PaletteFull)?;
        let allocation = PaletteAllocation::Multi(allocation);

        Ok(Self(RefCount::new_in(allocation, PaletteArena)))
    }

    #[must_use]
    /// Allocates the palette sharing an existing allocation where possible
    pub fn new(palette: &'static PaletteMulti) -> Self {
        unsafe {
            SPRITE_LOADER
                .palette(Palette::Multi(palette))
                .expect("palette unallocatable")
                .multi()
                .unwrap_unchecked()
        }
    }
}

/// A single or multi palette allocated to vram. This is reference counted and
/// cheap to clone.
#[derive(Clone, Debug)]
pub struct PaletteVram(RefCountedAllocation);

impl PaletteVram {
    /// Allocates a palette to vram. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn new_single(palette: &Palette16) -> Result<Self, LoaderError> {
        PaletteVramSingle::try_allocate_new(palette).map(PaletteVramSingle::palette)
    }

    /// Allocates a palette to vram. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn new_multi(palette: &PaletteMulti) -> Result<Self, LoaderError> {
        PaletteVramMulti::try_allocate_new(palette).map(PaletteVramMulti::palette)
    }

    /// If possible gets the 16 colour palette stored in this allocation
    pub fn single(self) -> Result<PaletteVramSingle, Self> {
        match &*self.0 {
            PaletteAllocation::Single(_) => Ok(PaletteVramSingle(self.0)),
            PaletteAllocation::Multi(_) => Err(self),
        }
    }

    /// If possible gets the multi palette stored in this allocation
    pub fn multi(self) -> Result<PaletteVramMulti, Self> {
        match &*self.0 {
            PaletteAllocation::Single(_) => Err(self),
            PaletteAllocation::Multi(_) => Ok(PaletteVramMulti(self.0)),
        }
    }

    #[must_use]
    pub(crate) fn strong_count(&self) -> usize {
        RefCount::count(&self.0)
    }

    #[must_use]
    pub(crate) fn is_multi(&self) -> bool {
        match &*self.0 {
            PaletteAllocation::Single(_) => false,
            PaletteAllocation::Multi(_) => true,
        }
    }

    #[must_use]
    pub(crate) fn single_palette_index(&self) -> Option<u8> {
        match &*self.0 {
            PaletteAllocation::Single(p) => Some(p.0),
            PaletteAllocation::Multi(_) => None,
        }
    }
}
