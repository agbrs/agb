use core::{alloc::Layout, cell::UnsafeCell, ptr::NonNull};

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

pub struct FixedSizeAllocatorInner<const SIZE: usize> {
    inner: BumpAllocatorInner,
    first_free_block: Option<SendNonNull<Block>>,
}

impl<const SIZE: usize> FixedSizeAllocatorInner<SIZE> {
    fn alloc(&mut self) -> Option<NonNull<[u8; SIZE]>> {
        if let Some(free_block) = self.first_free_block {
            let block = free_block.0.as_ptr();
            self.first_free_block = unsafe { (*block).next };

            return NonNull::new(block.cast());
        }
        let layout = Block::block_or_size(SIZE);

        self.inner.alloc(layout).map(core::ptr::NonNull::cast)
    }

    unsafe fn dealloc(&mut self, ptr: *mut [u8; SIZE]) {
        let block: *mut Block = ptr.cast();

        unsafe { (*block).next = self.first_free_block };
        self.first_free_block = NonNull::new(block).map(SendNonNull);
    }

    const fn new(start_end: StartEnd) -> Self {
        FixedSizeAllocatorInner {
            inner: BumpAllocatorInner::new(start_end),
            first_free_block: None,
        }
    }
}

pub(crate) struct FixedSizeAllocator<const SIZE: usize> {
    inner: UnsafeCell<FixedSizeAllocatorInner<SIZE>>,
}

unsafe impl<const SIZE: usize> Sync for FixedSizeAllocator<SIZE> {}

impl<const SIZE: usize> FixedSizeAllocator<SIZE> {
    pub(crate) const unsafe fn new(start: StartEnd) -> Self {
        Self {
            inner: UnsafeCell::new(FixedSizeAllocatorInner::new(start)),
        }
    }

    #[inline(always)]
    unsafe fn with_inner<F, T>(&self, f: F) -> T
    where
        F: Fn(&mut FixedSizeAllocatorInner<SIZE>) -> T,
    {
        let inner = &mut *self.inner.get();

        f(inner)
    }

    pub unsafe fn alloc(&self) -> Option<NonNull<[u8; SIZE]>> {
        self.with_inner(FixedSizeAllocatorInner::alloc)
    }

    pub unsafe fn dealloc(&self, ptr: *mut [u8; SIZE]) {
        self.with_inner(|inner| inner.dealloc(ptr));
    }
}
