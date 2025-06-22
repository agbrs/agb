use core::{mem::MaybeUninit, ptr::NonNull};

use alloc::{boxed::Box, vec};

use crate::display::tiled::{CHARBLOCK_SIZE, VRAM_START};

use super::TileFormat;

const AFFINE_ALLOC_END: usize = VRAM_START + 256 * TileFormat::EightBpp.tile_size();

pub(crate) struct TileAllocator {
    affine_allocator: MaybeUninit<TileAllocatorInner>,
    regular_allocator: MaybeUninit<TileAllocatorInner>,
}

impl TileAllocator {
    /// SAFETY: Ensure you call `init()` before calling any other methods on this
    pub const unsafe fn new() -> Self {
        Self {
            affine_allocator: MaybeUninit::uninit(),
            regular_allocator: MaybeUninit::uninit(),
        }
    }

    /// SAFETY: Only call this once
    pub unsafe fn init(&mut self) {
        // Leave the first tile unallocated for usage as the blank tile.
        // The number of 4bpp tiles in the affine space is 2 * 256 because there are 256 affine tiles we can use.
        // Subtract 2 for the reserved ones.
        self.affine_allocator
            .write(unsafe { TileAllocatorInner::new((VRAM_START + 8 * 8) as *mut _, 256 * 2 - 2) });

        // We assign 2 charblocks total. The CHARBLOCK_SIZE is in bytes, so we need to convert that into 4bpp tiles
        self.regular_allocator.write(unsafe {
            TileAllocatorInner::new(
                AFFINE_ALLOC_END as *mut _,
                CHARBLOCK_SIZE * 2 / TileFormat::FourBpp.tile_size() - 256 * 2,
            )
        });
    }

    pub fn alloc_for_regular(&mut self, tile_format: TileFormat) -> NonNull<u32> {
        match self.alloc_in_regular(tile_format) {
            Some(ptr) => ptr,
            None => self
                .alloc_in_affine(tile_format)
                .expect("Ran out of video RAM for tiles"),
        }
    }

    pub fn alloc_for_affine(&mut self) -> NonNull<u32> {
        self.alloc_in_affine(TileFormat::EightBpp)
            .expect("Ran out of video RAM for affine tiles")
    }

    pub unsafe fn dealloc(&mut self, ptr: NonNull<u32>, tile_format: TileFormat) {
        let allocator = if ptr.addr().get() < AFFINE_ALLOC_END {
            unsafe { self.affine_allocator.assume_init_mut() }
        } else {
            unsafe { self.regular_allocator.assume_init_mut() }
        };

        unsafe {
            allocator.dealloc(ptr, tile_format);
        }
    }

    fn alloc_in_regular(&mut self, tile_format: TileFormat) -> Option<NonNull<u32>> {
        let ptr = unsafe { self.regular_allocator.assume_init_mut() }.allocate(tile_format)?;
        debug_assert!(ptr.addr().get() >= AFFINE_ALLOC_END);
        Some(ptr)
    }

    fn alloc_in_affine(&mut self, tile_format: TileFormat) -> Option<NonNull<u32>> {
        let ptr = unsafe { self.affine_allocator.assume_init_mut() }.allocate(tile_format)?;
        debug_assert!(ptr.addr().get() < AFFINE_ALLOC_END);
        Some(ptr)
    }
}

#[derive(Debug)]
struct TileAllocatorInner {
    usage: Box<[u16]>,
    base_ptr: *const u32,

    first_unused_8bpp: Option<NonNull<Unused8BppBlock>>,
    first_unused_4bpp: Option<NonNull<Unused4BppBlock>>,
}

struct Unused8BppBlock {
    next: Option<NonNull<Unused8BppBlock>>,
}

#[derive(Clone)]
struct Unused4BppBlock {
    next: Option<NonNull<Unused4BppBlock>>,
    prev: Option<NonNull<Unused4BppBlock>>,
}

impl TileAllocatorInner {
    unsafe fn new(base_ptr: *mut u8, n_4bpp_tiles: usize) -> Self {
        assert_eq!(
            n_4bpp_tiles % 2,
            0,
            "n_4bpp_tiles must be even, got {n_4bpp_tiles}"
        );

        let usage = vec![0; n_4bpp_tiles.div_ceil(16)];

        let first_unused_8bpp = unsafe { fill_in_unused_chunks(base_ptr, n_4bpp_tiles) };

        Self {
            usage: usage.into_boxed_slice(),
            base_ptr: base_ptr.cast_const().cast(),

            first_unused_8bpp,
            first_unused_4bpp: None,
        }
    }

    fn allocate(&mut self, tile_format: TileFormat) -> Option<NonNull<u32>> {
        match tile_format {
            TileFormat::FourBpp => self.allocate_4bpp(),
            TileFormat::EightBpp => self.allocate_8bpp(),
        }
    }

    unsafe fn dealloc(&mut self, block: NonNull<u32>, tile_format: TileFormat) {
        unsafe {
            match tile_format {
                TileFormat::FourBpp => self.dealloc_4bpp(block),
                TileFormat::EightBpp => self.dealloc_8bpp(block),
            }
        }
    }

    fn allocate_8bpp(&mut self) -> Option<NonNull<u32>> {
        let first = self.first_unused_8bpp?;

        self.first_unused_8bpp = unsafe { (*first.as_ptr()).next };

        Some(first.cast())
    }

    unsafe fn dealloc_8bpp(&mut self, block: NonNull<u32>) {
        let next = self.first_unused_8bpp;

        let new_block = Unused8BppBlock { next };
        unsafe { *block.as_ptr().cast() = new_block };

        self.first_unused_8bpp = Some(block.cast());
    }

    fn allocate_4bpp(&mut self) -> Option<NonNull<u32>> {
        let next_block = if let Some(next_4bpp) = self.first_unused_4bpp {
            self.first_unused_4bpp = unsafe { Self::pop_4bpp(next_4bpp) };

            next_4bpp
        } else {
            // We need to split an 8bpp block into 2 4bpp blocks
            let next_8bpp = self.allocate_8bpp()?;

            // take the second half and call that a 4bpp tile
            let second_4bpp = unsafe { next_8bpp.byte_add(TileFormat::FourBpp.tile_size()) };

            // We know this is the only one because otherwise the other branch would've been taken
            let unused_block_for_second = Unused4BppBlock {
                next: None,
                prev: None,
            };

            unsafe {
                *second_4bpp.as_ptr().cast() = unused_block_for_second;
            }

            self.first_unused_4bpp = Some(second_4bpp.cast());
            next_8bpp.cast()
        };

        // Mark this tile as used
        let usage = self.get_usage_index_mask(next_block.cast());
        self.usage[usage.index()] |= usage.mask();

        Some(next_block.cast())
    }

    unsafe fn dealloc_4bpp(&mut self, block: NonNull<u32>) {
        let usage = self.get_usage_index_mask(block);
        self.usage[usage.index()] &= !usage.mask();

        let buddy = usage.buddy();

        if (self.usage[buddy.index()] & buddy.mask()) != 0 {
            // easy case because the buddy is used so just add `block` to the unused list
            let new_unused_block = Unused4BppBlock {
                next: self.first_unused_4bpp,
                prev: None,
            };

            if let Some(first_unused_4bpp) = self.first_unused_4bpp {
                unsafe { (*first_unused_4bpp.as_ptr()).prev = Some(block.cast()) };
            }

            unsafe {
                *block.as_ptr().cast() = new_unused_block;
            }

            self.first_unused_4bpp = Some(block.cast());
        } else {
            // Hard case. We want to combine this block and its buddy to form a brand new 8bpp block.

            // Step 1. Remove the buddy from the list
            let buddy_ptr = buddy.ptr(self.base_ptr);

            let buddy_unused_block =
                unsafe { (*buddy_ptr.as_ptr().cast::<Unused4BppBlock>()).clone() };

            if let Some(buddy_previous) = buddy_unused_block.prev {
                unsafe {
                    (*buddy_previous.as_ptr()).next = buddy_unused_block.next;
                }
            } else {
                // if the buddy's previous value is null, then it _is_ the first free slot, so
                // we should update the current free slot to the buddy's next slot
                self.first_unused_4bpp = buddy_unused_block.next;
            }

            if let Some(buddy_next) = buddy_unused_block.next {
                unsafe {
                    (*buddy_next.as_ptr()).prev = buddy_unused_block.prev;
                }
            }

            // Step 2. Make this one an 8bpp block because we're now one of these
            let eight_bpp_block = usage.eight_bpp_block().ptr(self.base_ptr);

            unsafe {
                self.dealloc_8bpp(eight_bpp_block);
            }
        }
    }

    fn get_usage_index_mask(&self, block: NonNull<u32>) -> UsageMaskIndex {
        let four_bpp_index =
            (block.as_ptr() as usize - self.base_ptr as usize) / TileFormat::FourBpp.tile_size();

        UsageMaskIndex(four_bpp_index)
    }

    // Fixes the next one and returns the new next
    // Can only be used for the first entry (i.e. prev is None)
    unsafe fn pop_4bpp(
        four_bpp_block: NonNull<Unused4BppBlock>,
    ) -> Option<NonNull<Unused4BppBlock>> {
        unsafe {
            debug_assert!((*four_bpp_block.as_ptr()).prev.is_none());
        }

        let next_entry = unsafe { (*four_bpp_block.as_ptr()).next }?;

        unsafe {
            (*next_entry.as_ptr()).prev = None;
        }

        Some(next_entry)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UsageMaskIndex(usize);

impl UsageMaskIndex {
    fn mask(self) -> u16 {
        1 << (self.0 % 16)
    }

    fn buddy(self) -> Self {
        Self(self.0 ^ 1)
    }

    fn index(self) -> usize {
        self.0 / 16
    }

    fn ptr(self, base_ptr: *const u32) -> NonNull<u32> {
        let ptr = base_ptr.wrapping_byte_add(self.0 * TileFormat::FourBpp.tile_size());

        NonNull::new(ptr.cast_mut()).unwrap()
    }

    fn eight_bpp_block(self) -> Self {
        Self(self.0 & !1)
    }
}

unsafe fn fill_in_unused_chunks(
    base_ptr: *mut u8,
    n_4bpp_tiles: usize,
) -> Option<NonNull<Unused8BppBlock>> {
    let mut next = None;
    for i in (0..n_4bpp_tiles / 2).rev() {
        let this_ptr: NonNull<Unused8BppBlock> = NonNull::new(
            base_ptr
                .wrapping_byte_add(i * TileFormat::EightBpp.tile_size())
                .cast(),
        )
        .unwrap();

        let unused_block = Unused8BppBlock { next };

        unsafe {
            *this_ptr.as_ptr() = unused_block;
        }

        next = Some(this_ptr);
    }

    next
}

#[cfg(test)]
mod test {
    use alloc::slice;

    use crate::{Gba, rng};

    use super::*;

    #[test_case]
    fn initialisation(_: &mut Gba) {
        core::hint::black_box(AllocatorTest::new(8));
    }

    #[test_case]
    fn allocate_some_4bpp_tiles(_: &mut Gba) {
        let mut allocator = AllocatorTest::new(8);

        let first_tile = allocator.allocate_4bpp().unwrap();
        let second_tile = allocator.allocate_4bpp().unwrap();

        let umi1 = allocator.allocator.get_usage_index_mask(first_tile);
        let umi2 = allocator.allocator.get_usage_index_mask(second_tile);

        assert_eq!(umi1.0, 0);
        assert_eq!(umi2.0, 1);

        assert_eq!(umi1.ptr(allocator.allocator.base_ptr), first_tile);
        assert_eq!(umi1.buddy(), umi2);
    }

    #[test_case]
    fn allocator_and_deallocate_first_4bpp_tiles(_: &mut Gba) {
        let mut allocator = AllocatorTest::new(8);

        let first_tile = allocator.allocate_4bpp().unwrap();
        let _second_tile = allocator.allocate_4bpp().unwrap();

        unsafe {
            allocator.allocator.dealloc_4bpp(first_tile);
        }

        let first_tile2 = allocator.allocate_4bpp().unwrap();

        assert_eq!(first_tile, first_tile2);
    }

    #[test_case]
    fn allocator_and_deallocate_first_4bpp_tiles(_: &mut Gba) {
        let mut allocator = AllocatorTest::new(8);

        let _first_tile = allocator.allocate_4bpp().unwrap();
        let second_tile = allocator.allocate_4bpp().unwrap();

        unsafe {
            allocator.allocator.dealloc_4bpp(second_tile);
        }

        let second_tile2 = allocator.allocate_4bpp().unwrap();

        assert_eq!(second_tile, second_tile2);
    }

    #[test_case]
    fn allocate_and_deallocate_to_merge(_: &mut Gba) {
        let mut allocator = AllocatorTest::new(8);

        let first_tile = allocator.allocate_4bpp().unwrap();
        let second_tile = allocator.allocate_4bpp().unwrap();

        unsafe {
            allocator.allocator.dealloc_4bpp(first_tile);
            allocator.allocator.dealloc_4bpp(second_tile);
        }

        let third_tile = allocator.allocate_8bpp().unwrap();

        assert_eq!(first_tile, third_tile);
    }

    #[test_case]
    fn allocate_and_deallocate_interleaved_fuzzed(_: &mut Gba) {
        let mut allocator = AllocatorTest::new(260);
        let mut tiles_4bpp = vec![];
        let mut tiles_8bpp = vec![];

        for _ in 0..1000 {
            match rng::next_i32().rem_euclid(4) {
                0 => {
                    if let Some(four_bpp_tile) = allocator.allocate_4bpp() {
                        tiles_4bpp.push(four_bpp_tile);
                    }
                }
                1 => {
                    if let Some(eight_bpp_tile) = allocator.allocate_8bpp() {
                        tiles_8bpp.push(eight_bpp_tile);
                    }
                }
                2 => {
                    if !tiles_4bpp.is_empty() {
                        let random = tiles_4bpp.swap_remove(
                            rng::next_i32().rem_euclid(tiles_4bpp.len() as i32) as usize,
                        );

                        unsafe {
                            allocator.allocator.dealloc_4bpp(random);
                        }
                    }
                }
                3 => {
                    if !tiles_8bpp.is_empty() {
                        let random = tiles_8bpp.swap_remove(
                            rng::next_i32().rem_euclid(tiles_8bpp.len() as i32) as usize,
                        );

                        unsafe {
                            allocator.allocator.dealloc_8bpp(random);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    struct AllocatorTest {
        _allocator_space: Box<[u32]>,
        allocator: TileAllocatorInner,
    }

    impl AllocatorTest {
        fn new(four_bpp_tiles: usize) -> Self {
            let vec_length = TileFormat::FourBpp.tile_size() * four_bpp_tiles / size_of::<u32>();
            let space = vec![0u32; vec_length].into_boxed_slice();

            Self {
                allocator: unsafe {
                    TileAllocatorInner::new(space.as_ptr().cast_mut().cast(), four_bpp_tiles)
                },
                _allocator_space: space,
            }
        }

        fn allocate_4bpp(&mut self) -> Option<NonNull<u32>> {
            let tile = self.allocator.allocate_4bpp()?;
            unsafe {
                fill_tile(tile, TileFormat::FourBpp);
            }

            Some(tile)
        }

        fn allocate_8bpp(&mut self) -> Option<NonNull<u32>> {
            let tile = self.allocator.allocate_8bpp()?;
            unsafe {
                fill_tile(tile, TileFormat::EightBpp);
            }

            Some(tile)
        }
    }

    unsafe fn fill_tile(block: NonNull<u32>, format: TileFormat) {
        unsafe { slice::from_raw_parts_mut(block.as_ptr().cast::<u8>(), format.tile_size()) }
            .fill(0x77);
    }
}
