use core::cell::UnsafeCell;

pub struct SyncUnsafeCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for SyncUnsafeCell<T> {}
unsafe impl<T> Send for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    pub const fn new(t: T) -> Self {
        Self(UnsafeCell::new(t))
    }

    pub unsafe fn get(&self) -> *mut T {
        self.0.get()
    }
}
