use core::alloc::{GlobalAlloc, Layout};

use core::cell::RefCell;
use core::convert::TryInto;
use core::ptr::NonNull;

use crate::interrupt::free;
use bare_metal::{CriticalSection, Mutex};

use super::bump_allocator::BumpAllocator;
use super::SendNonNull;

struct Block {
    size: usize,
    next: Option<SendNonNull<Block>>,
}

impl Block {
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
    }
}

struct BlockAllocatorState {
    first_free_block: Option<SendNonNull<Block>>,
}

pub(crate) struct BlockAllocator {
    inner_allocator: BumpAllocator,
    state: Mutex<RefCell<BlockAllocatorState>>,
}

impl BlockAllocator {
    pub(super) const unsafe fn new() -> Self {
        Self {
            inner_allocator: BumpAllocator::new(),
            state: Mutex::new(RefCell::new(BlockAllocatorState {
                first_free_block: None,
            })),
        }
    }

    #[allow(dead_code)]
    pub unsafe fn number_of_blocks(&self) -> u32 {
        free(|key| {
            let mut state = self.state.borrow(*key).borrow_mut();

            let mut count = 0;

            let mut list_ptr = &mut state.first_free_block;
            while let Some(mut curr) = list_ptr {
                count += 1;
                list_ptr = &mut curr.as_mut().next;
            }

            count
        })
    }

    fn new_block(&self, layout: Layout, cs: &CriticalSection) -> *mut u8 {
        let overall_layout = Block::either_layout(layout);
        self.inner_allocator.alloc_critical(overall_layout, cs)
    }

    unsafe fn normalise(&self) {
        free(|key| {
            let mut state = self.state.borrow(*key).borrow_mut();

            let mut list_ptr = &mut state.first_free_block;

            while let Some(mut curr) = list_ptr {
                if let Some(next_elem) = curr.as_mut().next {
                    let difference = next_elem
                        .as_ptr()
                        .cast::<u8>()
                        .offset_from(curr.as_ptr().cast::<u8>());
                    let usize_difference: usize = difference
                        .try_into()
                        .expect("distances in alloc'd blocks must be positive");

                    if usize_difference == curr.as_mut().size {
                        let current = curr.as_mut();
                        let next = next_elem.as_ref();

                        current.size += next.size;
                        current.next = next.next;
                    }
                }
                list_ptr = &mut curr.as_mut().next;
            }
        });
    }
}

unsafe impl GlobalAlloc for BlockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // find a block that this current request fits in
        let full_layout = Block::either_layout(layout);

        let (block_after_layout, block_after_layout_offset) = full_layout
            .extend(Layout::new::<Block>().align_to(8).unwrap().pad_to_align())
            .unwrap();

        free(|key| {
            let mut state = self.state.borrow(*key).borrow_mut();
            let mut current_block = state.first_free_block;
            let mut list_ptr = &mut state.first_free_block;
            while let Some(mut curr) = current_block {
                let curr_block = curr.as_mut();
                if curr_block.size == full_layout.size() {
                    *list_ptr = curr_block.next;
                    return curr.as_ptr().cast();
                } else if curr_block.size >= block_after_layout.size() {
                    // can split block
                    let split_block = Block {
                        size: curr_block.size - block_after_layout_offset,
                        next: curr_block.next,
                    };
                    let split_ptr = curr
                        .as_ptr()
                        .cast::<u8>()
                        .add(block_after_layout_offset)
                        .cast();
                    *split_ptr = split_block;
                    *list_ptr = NonNull::new(split_ptr).map(SendNonNull);

                    return curr.as_ptr().cast();
                }
                current_block = curr_block.next;
                list_ptr = &mut curr_block.next;
            }

            self.new_block(layout, key)
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let new_layout = Block::either_layout(layout).pad_to_align();
        free(|key| {
            let mut state = self.state.borrow(*key).borrow_mut();

            let mut list_ptr = &mut state.first_free_block;

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
        });
        self.normalise();
    }
}
