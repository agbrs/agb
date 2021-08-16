use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

use crate::interrupt::Mutex;

fn get_data_end() -> usize {
    extern "C" {
        static __ewram_data_end: usize;
    }

    // TODO: This seems completely wrong, but without the &, rust generates
    // a double dereference :/. Maybe a bug in nightly?
    (unsafe { &__ewram_data_end }) as *const _ as usize
}

pub(crate) struct BumpAllocator {
    current_ptr: Mutex<Option<NonNull<u8>>>,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            current_ptr: Mutex::new(None),
        }
    }
}

impl BumpAllocator {
    fn alloc_safe(&self, layout: Layout) -> *mut u8 {
        let mut current_ptr = self.current_ptr.lock();

        let ptr = if let Some(c) = *current_ptr {
            c.as_ptr() as usize
        } else {
            get_data_end()
        };

        let alignment_bitmask = layout.align() - 1;
        let fixup = ptr & alignment_bitmask;

        let amount_to_add = layout.align() - fixup;

        let resulting_ptr = ptr + amount_to_add;
        let new_current_ptr = resulting_ptr + layout.size();

        if new_current_ptr as usize >= super::EWRAM_END {
            return core::ptr::null_mut();
        }

        *current_ptr = NonNull::new(new_current_ptr as *mut _);

        resulting_ptr as *mut _
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_safe(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
