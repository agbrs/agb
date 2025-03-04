use core::{alloc::Layout, cell::Cell, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use alloc::alloc::Allocator;

struct NextFree {
    next: Option<NonNull<NextFree>>,
}

pub struct SingleAllocator<A: Allocator, T> {
    underlying: A,
    head: Cell<Option<NonNull<NextFree>>>,
    _phantom: PhantomData<T>,
}

impl<A: Allocator, T> SingleAllocator<A, T> {
    pub(crate) const fn new(allocator: A) -> Self {
        Self {
            underlying: allocator,
            head: Cell::new(None),
            _phantom: PhantomData,
        }
    }
}

unsafe impl<A: Allocator, T> Sync for SingleAllocator<A, T> {}

union NextOrType<T> {
    _t: ManuallyDrop<T>,
    _next: ManuallyDrop<NextFree>,
}

unsafe impl<A: Allocator, T> Allocator for SingleAllocator<A, T> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        assert_eq!(layout, Layout::new::<T>());

        let layout = Layout::new::<NextOrType<T>>();

        if let Some(next) = self.head.take() {
            self.head.set(unsafe { (*next.as_ptr()).next });
            Ok(unsafe {
                NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                    next.as_ptr().cast(),
                    layout.size(),
                ))
            })
        } else {
            self.underlying.allocate(layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        let this: NonNull<NextFree> = ptr.cast();
        unsafe { (*this.as_ptr()).next = self.head.take() };
        self.head.set(Some(this));
    }
}

macro_rules! create_allocator_arena {
    ($name: ident, $underlying: tt, $t: ty) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $name;

        impl $name {
            fn allocator()
            -> &'static $crate::agb_alloc::single_allocator::SingleAllocator<$underlying, $t> {
                static ALLOCATOR: $crate::agb_alloc::single_allocator::SingleAllocator<
                    $underlying,
                    $t,
                > = $crate::agb_alloc::single_allocator::SingleAllocator::new($underlying);
                &ALLOCATOR
            }
        }

        unsafe impl Allocator for $name {
            fn allocate(
                &self,
                layout: core::alloc::Layout,
            ) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
                Self::allocator().allocate(layout)
            }

            unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
                unsafe { Self::allocator().deallocate(ptr, layout) }
            }
        }
    };
}

pub(crate) use create_allocator_arena;
