use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

use crate::interrupt::Mutex;

use super::bump_allocator::BumpAllocator;

struct Block {
    used: bool,
    next: Option<NonNull<Block>>,
}

impl Block {
    pub unsafe fn from_data_ptr(data_ptr: *mut u8, layout: Layout) -> *mut Block {
        let block_layout = Layout::new::<Block>();
        let (_, offset) = block_layout.extend(layout).expect("Overflow on allocation");

        data_ptr.sub(offset).cast()
    }
}

struct BlockAllocatorState {
    first_block: Option<NonNull<Block>>,
    last_block: Option<NonNull<Block>>,
}

pub(crate) struct BlockAllocator {
    inner_allocator: BumpAllocator,
    state: Mutex<BlockAllocatorState>,
}

unsafe impl GlobalAlloc for BlockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let block_layout = Layout::new::<Block>();
        let (overall_layout, offset) = block_layout.extend(layout).expect("Overflow on allocation");

        let block_ptr = self.inner_allocator.alloc(overall_layout);

        if block_ptr.is_null() {
            return core::ptr::null_mut();
        }

        let block_ptr = NonNull::new_unchecked(block_ptr).cast();

        let mut state = self.state.lock();

        *block_ptr.cast::<Block>().as_mut() = Block {
            used: true,
            next: None,
        };

        state.first_block.get_or_insert(block_ptr);

        if let Some(last_block) = state.last_block {
            last_block.cast::<Block>().as_mut().next = Some(block_ptr);
        }
        state.last_block = Some(block_ptr);

        block_ptr.as_ptr().cast::<u8>().add(offset)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let block = Block::from_data_ptr(ptr, layout);
        (&mut *block).used = false;
    }
}
