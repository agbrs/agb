//! The block allocator works by maintaining a linked list of unused blocks and
//! requesting new blocks using a bump allocator. Freed blocks are inserted into
//! the linked list in order of pointer. Blocks are then merged after every
//! free.

use core::alloc::{Allocator, GlobalAlloc, Layout};

use core::cell::UnsafeCell;
use core::ptr::NonNull;

use super::bump_allocator::{BumpAllocatorInner, StartEnd};
use super::SendNonNull;

struct Block {
    size: usize,
    next: Option<SendNonNull<Block>>,
}

impl Block {
    /// Returns the layout of either the block or the wanted layout aligned to
    /// the maximum alignment used (double word).
    pub fn either_layout(layout: Layout) -> Layout {
        let block_layout = Layout::new::<Block>();
        let aligned_to = layout
            .align_to(block_layout.align())
            .expect("too large allocation");
        Layout::from_size_align(
            block_layout.size().max(aligned_to.size()),
            aligned_to.align(),
        )
        .expect("too large allocation")
        .align_to(8)
        .expect("too large allocation")
        .pad_to_align()
    }

    pub fn layout() -> Layout {
        Layout::new::<Block>().align_to(8).unwrap().pad_to_align()
    }
}

struct BlockAllocatorState {
    first_free_block: Option<SendNonNull<Block>>,
}

struct BlockAllocatorInner {
    inner_allocator: BumpAllocatorInner,
    state: BlockAllocatorState,
}

pub struct BlockAllocator {
    inner: UnsafeCell<BlockAllocatorInner>,
}

unsafe impl Sync for BlockAllocator {}

impl BlockAllocator {
    pub(crate) const unsafe fn new(start: StartEnd) -> Self {
        Self {
            inner: UnsafeCell::new(BlockAllocatorInner::new(start)),
        }
    }

    #[inline(always)]
    unsafe fn with_inner<F, T>(&self, f: F) -> T
    where
        F: Fn(&mut BlockAllocatorInner) -> T,
    {
        let inner = &mut *self.inner.get();

        f(inner)
    }

    #[doc(hidden)]
    #[cfg(any(test, feature = "testing"))]
    pub unsafe fn number_of_blocks(&self) -> u32 {
        self.with_inner(|inner| inner.number_of_blocks())
    }

    pub unsafe fn alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
        self.with_inner(|inner| inner.alloc(layout))
    }

    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.with_inner(|inner| inner.dealloc(ptr, layout));
    }

    pub unsafe fn grow(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout,
    ) -> Option<NonNull<u8>> {
        self.with_inner(|inner| inner.grow(ptr, layout, new_layout))
    }
}

impl BlockAllocatorInner {
    pub(crate) const unsafe fn new(start: StartEnd) -> Self {
        Self {
            inner_allocator: BumpAllocatorInner::new(start),
            state: BlockAllocatorState {
                first_free_block: None,
            },
        }
    }

    #[doc(hidden)]
    #[cfg(any(test, feature = "testing"))]
    pub unsafe fn number_of_blocks(&mut self) -> u32 {
        let mut count = 0;

        let mut list_ptr = &mut self.state.first_free_block;
        while let Some(mut current) = list_ptr {
            count += 1;
            list_ptr = &mut current.as_mut().next;
        }

        count
    }

    /// Requests a brand new block from the inner bump allocator
    fn new_block(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let overall_layout = Block::either_layout(layout);
        self.inner_allocator.alloc(overall_layout)
    }

    /// Merges blocks together to create a normalised list
    unsafe fn normalise(&mut self, point_to_normalise: *mut Block) {
        unsafe fn normalise_block(block_to_normalise: &mut Block) {
            if let Some(next_block) = block_to_normalise.next {
                let difference = next_block
                    .as_ptr()
                    .cast::<u8>()
                    .offset_from((block_to_normalise as *mut Block).cast::<u8>());
                if difference == block_to_normalise.size as isize {
                    let next = next_block.as_ref();
                    block_to_normalise.next = next.next;
                    block_to_normalise.size += next.size;
                    normalise_block(block_to_normalise);
                }
            }
        }

        normalise_block(&mut *point_to_normalise);
        if let Some(mut next_block) = (*point_to_normalise).next {
            normalise_block(next_block.as_mut());
        }
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        // find a block that this current request fits in
        let full_layout = Block::either_layout(layout);

        let mut list_ptr = &mut self.state.first_free_block;
        // This iterates the free list until it either finds a block that
        // is the exact size requested or a block that can be split into
        // one with the desired size and another block header.
        loop {
            match list_ptr {
                Some(mut current_block) => {
                    if let Some(alloc) = Self::allocate_into_block(list_ptr, full_layout) {
                        return Some(alloc);
                    }
                    list_ptr = &mut current_block.as_mut().next;
                }
                None => return self.new_block(layout),
            }
        }
    }

    /// splits a block in twain
    unsafe fn allocate_into_block(
        reference_to_block_pointer: &mut Option<SendNonNull<Block>>,
        wanted_layout: Layout,
    ) -> Option<NonNull<u8>> {
        let (extended_layout, offset) = wanted_layout.extend(Block::layout()).unwrap();

        let mut examination_block_ptr = reference_to_block_pointer.unwrap().0;
        let examination_block = examination_block_ptr.as_mut();

        if examination_block.size == wanted_layout.size() {
            *reference_to_block_pointer = examination_block.next;
            Some(examination_block_ptr.cast())
        } else if examination_block.size >= extended_layout.size() {
            let split_block = Block {
                size: examination_block.size - offset,
                next: examination_block.next,
            };

            let split_block_ptr = examination_block_ptr
                .as_ptr()
                .cast::<u8>()
                .add(offset)
                .cast();
            *split_block_ptr = split_block;
            *reference_to_block_pointer = NonNull::new(split_block_ptr).map(SendNonNull);

            Some(examination_block_ptr.cast())
        } else {
            None
        }
    }

    pub unsafe fn grow(
        &mut self,
        ptr: *mut u8,
        initial_layout: Layout,
        desired_layout: Layout,
    ) -> Option<NonNull<u8>> {
        let either_layout_initial = Block::either_layout(initial_layout);
        let either_layout_desired = Block::either_layout(desired_layout);

        let difference = Layout::from_size_align(
            either_layout_desired.size() - either_layout_initial.size(),
            either_layout_initial.align(),
        )
        .expect("should be able to construct difference layout");

        if self.is_block_at_end(ptr, either_layout_initial) {
            let _additional_space = self.inner_allocator.alloc(difference);
            return NonNull::new(ptr);
        }

        // cases
        // * Our block has no free block after it.
        // * Our block has a free block after that we fit in.
        // * Our block has a free block after that is too small.
        // * UNIMPLEMENTED Out block has a free block after that is too small but that is at the end so we can bump allocate some more space.

        let next_block = self.find_first_block_after(ptr);

        if let Some(list_to_block) = next_block {
            let is_block_directly_after = {
                if let Some(block) = list_to_block {
                    block.0.as_ptr() == ptr.add(either_layout_initial.size()).cast()
                } else {
                    false
                }
            };

            if is_block_directly_after {
                if let Some(_split) = Self::allocate_into_block(list_to_block, difference) {
                    return NonNull::new(ptr);
                }
            }
        }

        self.grow_copy(ptr, either_layout_initial, either_layout_desired)
    }

    unsafe fn grow_copy(
        &mut self,
        ptr: *mut u8,
        initial_layout: Layout,
        desired_layout: Layout,
    ) -> Option<NonNull<u8>> {
        let new_ptr = self.alloc(desired_layout)?;

        core::ptr::copy_nonoverlapping(ptr, new_ptr.as_ptr(), initial_layout.size());
        self.dealloc(ptr, initial_layout);

        Some(new_ptr)
    }

    unsafe fn is_block_at_end(&self, ptr: *mut u8, total_layout: Layout) -> bool {
        self.inner_allocator.tip() == NonNull::new(ptr.add(total_layout.size()))
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let point_to_normalise = self.dealloc_no_normalise(ptr, layout);
        if let Some(block_to_normalise) = *point_to_normalise {
            self.normalise(block_to_normalise.as_ptr());
        }
    }

    /// Returns a reference to the pointer to the next block
    /// Useful because you can modify what points to the block and access the block
    unsafe fn find_first_block_after(
        &mut self,
        ptr: *mut u8,
    ) -> Option<&mut Option<SendNonNull<Block>>> {
        let mut list_ptr = &mut self.state.first_free_block;

        loop {
            match list_ptr {
                Some(mut current_block) => {
                    if current_block.as_ptr().cast() > ptr {
                        return Some(list_ptr);
                    }

                    list_ptr = &mut current_block.as_mut().next;
                }
                None => return None,
            }
        }
    }

    pub unsafe fn dealloc_no_normalise(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
    ) -> *mut Option<SendNonNull<Block>> {
        let new_layout = Block::either_layout(layout).pad_to_align();

        // note that this is a reference to a pointer
        let mut list_ptr = &mut self.state.first_free_block;
        let mut list_ptr_prev: *mut Option<SendNonNull<Block>> = list_ptr;

        // This searches the free list until it finds a block further along
        // than the block that is being freed. The newly freed block is then
        // inserted before this block. If the end of the list is reached
        // then the block is placed at the end with no new block after it.
        loop {
            match list_ptr {
                Some(mut current_block) => {
                    if current_block.as_ptr().cast() > ptr {
                        let new_block_content = Block {
                            size: new_layout.size(),
                            next: Some(current_block),
                        };
                        *ptr.cast() = new_block_content;
                        *list_ptr = NonNull::new(ptr.cast()).map(SendNonNull);
                        break;
                    }
                    list_ptr_prev = list_ptr;
                    list_ptr = &mut current_block.as_mut().next;
                }
                None => {
                    // reached the end of the list without finding a place to insert the value
                    let new_block_content = Block {
                        size: new_layout.size(),
                        next: None,
                    };
                    *ptr.cast() = new_block_content;
                    *list_ptr = NonNull::new(ptr.cast()).map(SendNonNull);
                    break;
                }
            }
        }

        list_ptr_prev
    }
}

unsafe impl GlobalAlloc for BlockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.alloc(layout) {
            None => core::ptr::null_mut(),
            Some(p) => p.as_ptr(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());

        if new_size > layout.size() {
            return match self.grow(ptr, layout, new_layout) {
                Some(p) => p.as_ptr(),
                None => core::ptr::null_mut(),
            };
        }

        let new_ptr = GlobalAlloc::alloc(self, new_layout);
        if !new_ptr.is_null() {
            core::ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
            self.dealloc(ptr, layout);
        }
        new_ptr
    }
}

unsafe impl Allocator for BlockAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        match unsafe { self.alloc(layout) } {
            None => Err(core::alloc::AllocError),
            Some(p) => Ok(unsafe {
                NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                    p.as_ptr(),
                    layout.size(),
                ))
            }),
        }
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        match self.grow(ptr.as_ptr(), old_layout, new_layout) {
            Some(p) => Ok(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                p.as_ptr(),
                new_layout.size(),
            ))),
            None => Err(core::alloc::AllocError),
        }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let new_ptr = self
            .grow(ptr.as_ptr(), old_layout, new_layout)
            .ok_or(core::alloc::AllocError)?;

        new_ptr
            .as_ptr()
            .add(old_layout.size())
            .write_bytes(0, new_layout.size() - old_layout.size());

        Ok(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
            new_ptr.as_ptr(),
            new_layout.size(),
        )))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.dealloc(ptr.as_ptr(), layout);
    }
}
