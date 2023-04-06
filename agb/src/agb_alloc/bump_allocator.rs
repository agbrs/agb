use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::NonNull;

use super::SendNonNull;
use crate::interrupt::free;
use bare_metal::Mutex;

pub(crate) struct StartEnd {
    pub start: fn() -> usize,
    pub end: fn() -> usize,
}

pub(crate) struct BumpAllocatorInner {
    current_ptr: Option<SendNonNull<u8>>,
    start_end: StartEnd,
}

pub(crate) struct BumpAllocator {
    inner: Mutex<RefCell<BumpAllocatorInner>>,
}

impl BumpAllocatorInner {
    pub const fn new(start_end: StartEnd) -> Self {
        Self {
            current_ptr: None,
            start_end,
        }
    }

    pub fn tip(&self) -> Option<NonNull<u8>> {
        self.current_ptr.map(|x| x.0)
    }

    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let current_ptr = &mut self.current_ptr;

        let ptr = if let Some(c) = *current_ptr {
            c.as_ptr() as usize
        } else {
            (self.start_end.start)()
        };

        let alignment_bitmask = layout.align() - 1;
        let fixup = ptr & alignment_bitmask;

        let amount_to_add = (layout.align() - fixup) & alignment_bitmask;

        let resulting_ptr = ptr + amount_to_add;
        let new_current_ptr = resulting_ptr + layout.size();

        if new_current_ptr >= (self.start_end.end)() {
            return None;
        }

        *current_ptr = NonNull::new(new_current_ptr as *mut _).map(SendNonNull);

        NonNull::new(resulting_ptr as *mut _)
    }
}

impl BumpAllocator {
    fn alloc_safe(&self, layout: Layout) -> Option<NonNull<u8>> {
        free(|key| self.inner.borrow(key).borrow_mut().alloc(layout))
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.alloc_safe(layout) {
            None => core::ptr::null_mut(),
            Some(p) => p.as_ptr(),
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
