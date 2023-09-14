mod locks;
mod statics;

pub use locks::*;
pub use statics::*;

use core::arch::asm;

/// Marks that a pointer is read without actually reading from this.
///
/// This uses an [`asm!`] instruction that marks the parameter as being read,
/// requiring the compiler to treat this function as if anything could be
/// done to it.
#[inline(always)]
pub fn memory_read_hint<T>(val: *const T) {
    unsafe { asm!("/* {0} */", in(reg) val, options(readonly, nostack)) }
}

/// Marks that a pointer is read or written to without actually writing to it.
///
/// This uses an [`asm!`] instruction that marks the parameter as being read
/// and written, requiring the compiler to treat this function as if anything
/// could be done to it.
#[inline(always)]
pub fn memory_write_hint<T>(val: *mut T) {
    unsafe { asm!("/* {0} */", in(reg) val, options(nostack)) }
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
