//! A reimplementation of Rc but with the inner type exposed

use core::{cell::Cell, fmt::Debug, ops::Deref, ptr::NonNull};

use alloc::{alloc::Allocator, boxed::Box};

pub struct RefCount<T, A: Allocator>(NonNull<RefCountInner<T>>, A);

pub struct RefCountInner<T> {
    count: Cell<usize>,
    inner: T,
}

impl<T, A: Allocator> RefCount<T, A> {
    fn inner(&self) -> &RefCountInner<T> {
        unsafe { self.0.as_ref() }
    }

    pub fn count(s: &Self) -> usize {
        s.inner().count()
    }

    pub fn new_in(value: T, a: A) -> Self {
        let v = unsafe {
            NonNull::new_unchecked(
                Box::into_raw_with_allocator(Box::new_in(
                    RefCountInner {
                        inner: value,
                        count: Cell::new(1),
                    },
                    &a,
                ))
                .0,
            )
        };
        Self(v, a)
    }
}

impl<T: Debug, A: Allocator> Debug for RefCount<T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T> RefCountInner<T> {
    fn inc(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn dec(&self) {
        self.count.set(self.count.get() - 1);
    }

    fn count(&self) -> usize {
        self.count.get()
    }
}

impl<T, A: Allocator> Deref for RefCount<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().inner
    }
}

impl<T, A> Clone for RefCount<T, A>
where
    A: Allocator + Clone,
{
    fn clone(&self) -> Self {
        self.inner().inc();
        Self(self.0, self.1.clone())
    }
}

impl<T, A: Allocator> Drop for RefCount<T, A> {
    fn drop(&mut self) {
        self.inner().dec();
        if self.inner().count() == 0 {
            drop(unsafe { Box::from_non_null_in(self.0, &self.1) });
        }
    }
}
