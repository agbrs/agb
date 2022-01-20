use core::alloc::Layout;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

mod block_allocator;
mod bump_allocator;

use block_allocator::BlockAllocator;

struct SendNonNull<T>(NonNull<T>);
unsafe impl<T> Send for SendNonNull<T> {}

impl<T> Clone for SendNonNull<T> {
    fn clone(&self) -> Self {
        SendNonNull(self.0)
    }
}
impl<T> Copy for SendNonNull<T> {}

impl<T> Deref for SendNonNull<T> {
    type Target = NonNull<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SendNonNull<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

const EWRAM_END: usize = 0x0204_0000;

#[global_allocator]
static GLOBAL_ALLOC: BlockAllocator = unsafe { BlockAllocator::new() };

#[cfg(test)]
pub unsafe fn number_of_blocks() -> u32 {
    GLOBAL_ALLOC.number_of_blocks()
}

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
    use alloc::vec;
    use alloc::vec::Vec;

    #[test_case]
    fn test_box(_gba: &mut crate::Gba) {
        let first_box = Box::new(1);
        let second_box = Box::new(2);

        assert!(&*first_box as *const _ < &*second_box as *const _);
        assert_eq!(*first_box, 1);
        assert_eq!(*second_box, 2);

        let address = &*first_box as *const _ as usize;
        assert!(
            (EWRAM_START..EWRAM_END).contains(&address),
            "ewram is located between 0x0200_0000 and 0x0204_0000, address was actually found to be {:#010X}",
            address
        );
    }

    #[test_case]
    fn test_vec(_gba: &mut crate::Gba) {
        let mut v = Vec::with_capacity(5);

        for i in 0..100 {
            v.push(i);
        }

        for (i, &e) in v.iter().enumerate() {
            assert_eq!(e, i);
        }
    }

    #[test_case]
    fn test_creating_and_removing_things(_gba: &mut crate::Gba) {
        let item = Box::new(1);
        for i in 0..1_000 {
            let x = Box::new(i);
            assert_eq!(*x, i);
            let address = &*x as *const _ as usize;
            assert!(
                (EWRAM_START..EWRAM_END).contains(&address),
                "ewram is located between 0x0200_0000 and 0x0204_0000, address was actually found to be {:#010X}",
                address
            );
        }

        assert_eq!(*item, 1);
    }

    #[test_case]
    fn test_adding_to_2_different_vectors(_gba: &mut crate::Gba) {
        let mut v1 = vec![1, 2, 3];
        let mut v2 = vec![4, 5, 6];

        for i in 0..100 {
            v1.push(i + 100);
            v2.push(i + 1000);
        }

        assert_eq!(v1[40], 137);
        assert_eq!(v2[78], 1075);
    }
}
