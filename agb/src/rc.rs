use core::{
    alloc::Allocator, cell::Cell, marker::PhantomData, mem::MaybeUninit, ops::Deref, ptr::NonNull,
};

use alloc::{alloc::Global, boxed::Box};

struct BoxedRc<T> {
    strong: Cell<usize>,
    data: MaybeUninit<T>,
}

pub struct Rc<T, A: Allocator = Global> {
    inner: NonNull<BoxedRc<T>>,
    _phantom_drop: PhantomData<BoxedRc<T>>,
    allocator: A,
}

impl<T> Rc<T> {
    pub fn new(data: T) -> Self {
        Rc::new_in(data, Global)
    }
}

impl<T, A: Allocator> Rc<T, A> {
    fn inner(&self) -> &BoxedRc<T> {
        unsafe { self.inner.as_ref() }
    }

    pub fn new_in(data: T, allocator: A) -> Self {
        let boxed = BoxedRc {
            strong: Cell::new(1),
            data: MaybeUninit::new(data),
        };

        let in_box = Box::new_in(boxed, &allocator);
        let pointer = Box::into_raw(in_box);

        let non_null = unsafe { NonNull::new_unchecked(pointer) };

        Rc {
            inner: non_null,
            allocator,
            _phantom_drop: PhantomData,
        }
    }

    pub fn strong_count(&self) -> usize {
        self.inner().strong.get()
    }
}

fn deallocate_data<T>(mut inner: NonNull<BoxedRc<T>>) {
    unsafe { inner.as_mut().data.assume_init_drop() };
}

fn deallocate_inner<T, A: Allocator>(inner: NonNull<BoxedRc<T>>, allocator: A) {
    let _ = unsafe { Box::from_raw_in(inner.as_ptr(), allocator) };
}

impl<T, A: Allocator> Drop for Rc<T, A> {
    fn drop(&mut self) {
        let strong_count = {
            let r = self.inner();
            r.strong.set(r.strong.get() - 1);
            r.strong.get()
        };
        if strong_count == 0 {
            deallocate_data(self.inner);
            deallocate_inner(self.inner, &self.allocator);
        }
    }
}

impl<T, A: Allocator> Deref for Rc<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let r = self.inner();
        unsafe { r.data.assume_init_ref() }
    }
}

impl<T, A: Allocator + Clone> Clone for Rc<T, A> {
    fn clone(&self) -> Self {
        let r = self.inner();
        r.strong.set(r.strong.get() + 1);

        Self {
            inner: self.inner,
            allocator: self.allocator.clone(),
            _phantom_drop: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn test_rc(_gba: &mut crate::Gba) {
        let r = Rc::new(10);

        assert_eq!(*r, 10);

        {
            let _b = r.clone();

            assert_eq!(*r, 10);
            assert_eq!(_b.strong_count(), 2);
            assert_eq!(r.strong_count(), 2);
        }

        assert_eq!(r.strong_count(), 1);
    }
}
