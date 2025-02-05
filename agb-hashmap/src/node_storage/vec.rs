use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;

use crate::{Allocator, Global};

pub(crate) use inner::MyVec;

#[cfg(not(feature = "allocator_api"))]
mod inner {
    use super::*;

    #[derive(Clone)]
    pub(crate) struct MyVec<T, A: Allocator = Global>(Vec<T>, A);

    impl<T, A: Allocator> MyVec<T, A> {
        pub(crate) fn new_in(allocator: A) -> Self {
            Self(Vec::new(), allocator)
        }

        pub(crate) fn allocator(&self) -> &A {
            &self.1
        }
    }
    impl<T, A: Allocator> Deref for MyVec<T, A> {
        type Target = Vec<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T, A: Allocator> DerefMut for MyVec<T, A> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

#[cfg(feature = "allocator_api")]
mod inner {
    use super::*;

    #[derive(Clone)]
    pub(crate) struct MyVec<T, A: Allocator = Global>(Vec<T, A>);

    impl<T, A: Allocator> MyVec<T, A> {
        pub(crate) fn new_in(allocator: A) -> Self {
            Self(Vec::new_in(allocator))
        }
    }

    impl<T, A: Allocator> Deref for MyVec<T, A> {
        type Target = Vec<T, A>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T, A: Allocator> DerefMut for MyVec<T, A> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}
