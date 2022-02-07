use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::NonNull;

use super::SendNonNull;
use crate::interrupt::free;
use bare_metal::{CriticalSection, Mutex};

pub(crate) struct BumpAllocator {
    current_ptr: Mutex<RefCell<Option<SendNonNull<u8>>>>,
    start: Mutex<fn() -> usize>,
}

impl BumpAllocator {
    pub const fn new(start: fn() -> usize) -> Self {
        Self {
            current_ptr: Mutex::new(RefCell::new(None)),
            start: Mutex::new(start),
        }
    }
}

impl BumpAllocator {
    pub fn alloc_critical(&self, layout: Layout, cs: &CriticalSection) -> *mut u8 {
        let mut current_ptr = self.current_ptr.borrow(*cs).borrow_mut();

        let ptr = if let Some(c) = *current_ptr {
            c.as_ptr() as usize
        } else {
            self.start.borrow(*cs)()
        };

        let alignment_bitmask = layout.align() - 1;
        let fixup = ptr & alignment_bitmask;

        let amount_to_add = (layout.align() - fixup) & alignment_bitmask;

        let resulting_ptr = ptr + amount_to_add;
        let new_current_ptr = resulting_ptr + layout.size();

        if new_current_ptr as usize >= super::EWRAM_END {
            return core::ptr::null_mut();
        }

        *current_ptr = NonNull::new(new_current_ptr as *mut _).map(SendNonNull);

        resulting_ptr as *mut _
    }
    pub fn alloc_safe(&self, layout: Layout) -> *mut u8 {
        free(|key| self.alloc_critical(layout, key))
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_safe(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
