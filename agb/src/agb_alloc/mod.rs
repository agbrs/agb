use core::alloc::{GlobalAlloc, Layout};

use super::interrupt::Mutex;

fn get_data_end() -> usize {
    extern "C" {
        static __ewram_data_end: usize;
    }

    // TODO: This seems completely wrong, but without the &, rust generates
    // a double dereference :/. Maybe a bug in nightly?
    (unsafe { &__ewram_data_end }) as *const _ as usize
}

struct BumpAllocator {
    current_ptr: Mutex<*mut u8>,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            current_ptr: Mutex::new(core::ptr::null_mut()),
        }
    }
}

impl BumpAllocator {
    fn alloc_safe(&self, layout: Layout) -> *mut u8 {
        let mut current_ptr = self.current_ptr.lock();

        let ptr = if current_ptr.is_null() {
            get_data_end()
        } else {
            *current_ptr as usize
        };

        let alignment_bitmask = layout.align() - 1;
        let fixup = ptr & alignment_bitmask;

        let amount_to_add = layout.align() - fixup;

        let resulting_ptr = ptr + amount_to_add;
        *current_ptr = (resulting_ptr + layout.size()) as *mut _;

        resulting_ptr as *mut _
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_safe(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!(
        "Failed to allocate size {} with alignment {}",
        layout.size(),
        layout.align()
    );
}

#[global_allocator]
static GLOBAL_ALLOC: BumpAllocator = BumpAllocator::new();

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    #[test_case]
    fn test_box(_gba: &mut crate::Gba) {
        let first_box = Box::new(1);
        let second_box = Box::new(2);

        assert!(&*first_box as *const _ < &*second_box as *const _);
        assert_eq!(*first_box, 1);
        assert_eq!(*second_box, 2);

        let address = &*first_box as *const _ as usize;
        assert!(
            address >= 0x0200_0000 && address < 0x0204_0000,
            "ewram is located between 0x0200_0000 and 0x0204_0000, address was actually found to be {:#010X}",
            address
        );
    }
}
