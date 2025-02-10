use core::{alloc::Allocator, cell::Cell, hint::assert_unchecked, ptr::NonNull};

use crate::{
    agb_alloc::single_allocator::create_allocator_arena,
    display::{object::PaletteMulti, palette16::Palette16},
    refcount::{RefCount, RefCountInner},
    ExternalAllocator,
};

use super::LoaderError;

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

/// A palette containing 16 colours that is currently allocated to vram. To use
/// this palette will require 4 bits per pixel.
#[derive(Clone, Debug)]
pub struct PaletteVramSingle(PaletteVram);

impl PaletteVramSingle {
    /// Gets the general PaletteVram that represents this palette. This is
    /// common to both single and multi palettes.
    #[must_use]
    pub fn palette(self) -> PaletteVram {
        self.0
    }

    /// Allocates a palette into vram from a palette. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn new(palette: &Palette16) -> Result<Self, LoaderError> {
        PaletteVram::new_single(palette).map(Self)
    }
}

/// A palette that can contain more than 16 colours allocated to vram. To use
/// this palette will require 8 bits per pixel.
#[derive(Clone, Debug)]
pub struct PaletteVramMulti(PaletteVram);

impl PaletteVramMulti {
    /// Gets the general palette that represents this palette. This is common to
    /// both single and multi palettes.
    #[must_use]
    pub fn palette(self) -> PaletteVram {
        self.0
    }

    /// Allocates a palette into vram from a palette. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn new(palette: &PaletteMulti) -> Result<Self, LoaderError> {
        PaletteVram::new_multi(palette).map(Self)
    }
}

/// A single or multi palette allocated to vram. This is reference counted and
/// cheap to clone.
#[derive(Clone, Debug)]
pub struct PaletteVram(RefCount<PaletteAllocation, PaletteArena>);

impl PaletteVram {
    /// Allocates a palette to vram. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn new_single(palette: &Palette16) -> Result<Self, LoaderError> {
        let allocation = PALETTE_ALLOCATOR
            .allocate_single(palette)
            .ok_or(LoaderError::PaletteFull)?;
        let allocation = PaletteAllocation::Single(allocation);

        Ok(Self(RefCount::new_in(allocation, PaletteArena)))
    }

    /// Allocates a palette to vram. Generally this is only
    /// useful for a dynamic palette as it performs no deduplication.
    pub fn new_multi(palette: &PaletteMulti) -> Result<Self, LoaderError> {
        let allocation = PALETTE_ALLOCATOR
            .allocate_multiple(palette)
            .ok_or(LoaderError::PaletteFull)?;
        let allocation = PaletteAllocation::Multi(allocation);

        Ok(Self(RefCount::new_in(allocation, PaletteArena)))
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
