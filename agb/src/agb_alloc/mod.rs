use core::alloc::Layout;

mod block_allocator;
mod bump_allocator;

use block_allocator::BlockAllocator;

const EWRAM_END: usize = 0x0204_0000;

#[global_allocator]
static GLOBAL_ALLOC: BlockAllocator = unsafe { BlockAllocator::new() };

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!(
        "Failed to allocate size {} with alignment {}",
        layout.size(),
        layout.align()
    );
}

#[cfg(test)]
mod test {
    const EWRAM_START: usize = 0x0200_0000;

    use super::*;
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
            address >= EWRAM_START && address < EWRAM_END,
            "ewram is located between 0x0200_0000 and 0x0204_0000, address was actually found to be {:#010X}",
            address
        );
    }
}
