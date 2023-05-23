use core::{alloc::Layout, ptr::NonNull};

use super::{
    bump_allocator::{BumpAllocatorInner, StartEnd},
    SendNonNull,
};

struct Block {
    next: Option<SendNonNull<Block>>,
}

impl Block {
    fn block_or_size(size: usize) -> Layout {
        let block_layout = Layout::new::<Block>();
        Layout::from_size_align(block_layout.size().max(size), 8).unwrap()
    }
}

pub(crate) struct FixedSizeAllocator<const SIZE: usize> {
    inner: BumpAllocatorInner,
    first_free_block: Option<SendNonNull<Block>>,
}

impl<const SIZE: usize> FixedSizeAllocator<SIZE> {
    pub(crate) fn alloc(&mut self) -> Option<NonNull<[u8; SIZE]>> {
        if let Some(free_block) = self.first_free_block {
            let block = free_block.0.as_ptr();
            self.first_free_block = unsafe { (*block).next };

            return NonNull::new(block.cast());
        }
        let layout = Block::block_or_size(SIZE);

        self.inner.alloc(layout).map(core::ptr::NonNull::cast)
    }

    pub(crate) fn dealloc(&mut self, ptr: *mut [u8; SIZE]) {
        let block: *mut Block = ptr.cast();

        unsafe { (*block).next = self.first_free_block };
        self.first_free_block = NonNull::new(block).map(SendNonNull);
    }

    pub(crate) fn new(start_end: StartEnd) -> Self {
        FixedSizeAllocator {
            inner: BumpAllocatorInner::new(start_end),
            first_free_block: None,
        }
    }
}
