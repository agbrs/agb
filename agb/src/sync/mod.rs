use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use portable_atomic::{AtomicBool, Ordering};

#[inline(never)]
fn already_locked() -> ! {
    panic!("IRQ and main thread are attempting to access the same Lock!")
}

/// A lock that prevents code from running in both an IRQ and normal code at
/// the same time.
///
/// Note that this does not support blocking like a typical mutex, and instead
/// mainly exists for memory safety reasons.
pub struct RawLock(AtomicBool);
impl RawLock {
    /// Creates a new lock.
    #[must_use]
    pub const fn new() -> Self {
        RawLock(AtomicBool::new(false))
    }

    /// Locks the lock and returns whether a lock was successfully acquired.
    fn raw_lock(&self) -> bool {
        if self.0.swap(true, Ordering::Acquire) {
            // value was already true, oops.
            false
        } else {
            // prevent any weird reordering, and continue
            true
        }
    }

    /// Unlocks the lock.
    fn raw_unlock(&self) {
        if !self.0.swap(false, Ordering::Release) {
            panic!("Internal error: Attempt to unlock a `RawLock` which is not locked.")
        }
    }

    /// Returns a guard for this lock, or `None` if there is another lock active.
    pub fn try_lock(&self) -> Option<RawLockGuard<'_>> {
        if self.raw_lock() {
            Some(RawLockGuard(self))
        } else {
            None
        }
    }
}
unsafe impl Send for RawLock {}
unsafe impl Sync for RawLock {}

/// A guard representing an active lock on an [`RawLock`].
pub struct RawLockGuard<'a>(&'a RawLock);
impl<'a> Drop for RawLockGuard<'a> {
    fn drop(&mut self) {
        self.0.raw_unlock();
    }
}

/// A lock that protects an object from being accessed in both an IRQ and
/// normal code at once.
///
/// Note that this does not support blocking like a typical mutex, and instead
/// mainly exists for memory safety reasons.
pub struct Lock<T> {
    raw: RawLock,
    data: UnsafeCell<T>,
}
impl<T> Lock<T> {
    /// Creates a new lock containing a given value.
    #[must_use]
    pub const fn new(t: T) -> Self {
        Lock {
            raw: RawLock::new(),
            data: UnsafeCell::new(t),
        }
    }

    /// Returns a guard for this lock, or panics if there is another lock active.
    pub fn lock(&self) -> LockGuard<'_, T> {
        self.try_lock().unwrap_or_else(|| already_locked())
    }

    /// Returns a guard for this lock or `None` if there is another lock active.
    pub fn try_lock(&self) -> Option<LockGuard<'_, T>> {
        if self.raw.raw_lock() {
            Some(LockGuard {
                underlying: self,
                ptr: self.data.get(),
            })
        } else {
            None
        }
    }
}
unsafe impl<T> Send for Lock<T> {}
unsafe impl<T> Sync for Lock<T> {}

/// A guard representing an active lock on an [`Lock`].
pub struct LockGuard<'a, T> {
    underlying: &'a Lock<T>,
    ptr: *mut T,
}
impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.underlying.raw.raw_unlock();
    }
}
impl<'a, T> Deref for LockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
impl<'a, T> DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

/// An internal function used as a temporary hack to get `compiler_fence`
/// working. While this call is not properly inlined, working is better than not
/// working at all.
///
/// This seems to be a problem caused by Rust issue #62256:
/// <https://github.com/rust-lang/rust/issues/62256>
///
/// # Safety
///
/// **WARNING FOR ANYONE WHO FINDS THIS**: This implementation will *only* be
/// correct on the GBA, and should not be used on any other platform. The GBA
/// is very old, and has no atomics to begin with - only a main thread and
/// interrupts. On any more recent CPU, this implementation is extremely
/// unlikely to be sound.
///
/// Not public API, obviously.
#[doc(hidden)]
#[deprecated]
#[allow(dead_code)]
#[no_mangle]
#[inline(always)]
pub unsafe extern "C" fn __sync_synchronize() {}
