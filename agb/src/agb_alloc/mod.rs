use core::alloc::Layout;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

pub(crate) mod block_allocator;
pub(crate) mod bump_allocator;

use block_allocator::BlockAllocator;

use self::bump_allocator::StartEnd;

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
const IWRAM_END: usize = 0x0300_8000;

#[global_allocator]
static GLOBAL_ALLOC: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: data_end,
        end: || EWRAM_END,
    })
};

pub static EWRAM_ALLOC: &BlockAllocator = &GLOBAL_ALLOC;
pub static IWRAM_ALLOC: &BlockAllocator = &__IWRAM_ALLOC;
static __IWRAM_ALLOC: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: iwram_data_end,
        end: || IWRAM_END,
    })
};

#[cfg(any(test, feature = "testing"))]
pub(crate) unsafe fn number_of_blocks() -> u32 {
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

fn iwram_data_end() -> usize {
    extern "C" {
        static __iwram_data_end: usize;
    }

    // TODO: This seems completely wrong, but without the &, rust generates
    // a double dereference :/. Maybe a bug in nightly?
    (unsafe { &__iwram_data_end }) as *const _ as usize + 0x200
}

fn data_end() -> usize {
    extern "C" {
        static __ewram_data_end: usize;
    }

    // TODO: This seems completely wrong, but without the &, rust generates
    // a double dereference :/. Maybe a bug in nightly?
    (unsafe { &__ewram_data_end }) as *const _ as usize
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

    #[test_case]
    fn should_return_data_end_somewhere_in_ewram(_gba: &mut crate::Gba) {
        let data_end = data_end();

        assert!(
            0x0200_0000 <= data_end,
            "data end should be bigger than 0x0200_0000, got {}",
            data_end
        );
        assert!(
            0x0204_0000 > data_end,
            "data end should be smaller than 0x0203_0000"
        );
    }

    #[test_case]
    fn should_return_data_end_somewhere_in_iwram(_gba: &mut crate::Gba) {
        let data_end = iwram_data_end();

        assert!(
            (0x0300_0000..0x0300_8000).contains(&data_end),
            "iwram data end should be in iwram, instead was {}",
            data_end
        );
        crate::println!("data end was {:#010X}", data_end);
    }

    #[test_case]
    fn allocate_to_iwram_works(_gba: &mut crate::Gba) {
        let a = Box::new_in(1, IWRAM_ALLOC);
        let p = &*a as *const i32;
        let addr = p as usize;
        assert!(
            (0x0300_0000..0x0300_8000).contains(&addr),
            "address of allocation should be within iwram, instead at {:?}",
            p
        );
    }
}
