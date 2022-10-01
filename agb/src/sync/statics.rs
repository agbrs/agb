use core::arch::asm;
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;

/// The internal function for replacing a `Copy` (really `!Drop`) value in a
/// [`Static`]. This uses assembly to use an `stmia` instruction to ensure
/// an IRQ cannot occur during the write operation.
unsafe fn transfer<T: Copy>(dst: *mut T, src: *const T) {
    let align = mem::align_of::<T>();
    let size = mem::size_of::<T>();

    if size == 0 {
        // Do nothing with ZSTs.
    } else if size <= 16 && align % 4 == 0 {
        // We can do an 4-byte aligned transfer up to 16 bytes.
        transfer_align4_thumb(dst, src);
    } else if size <= 40 && align % 4 == 0 {
        // We can do the same up to 40 bytes, but we need to switch to ARM.
        transfer_align4_arm(dst, src);
    } else if size <= 2 && align % 2 == 0 {
        // We can do a 2-byte aligned transfer up to 2 bytes.
        asm!(
            "ldrh {2},[{0}]",
            "strh {2},[{1}]",
            in(reg) src, in(reg) dst, out(reg) _,
        );
    } else if size == 1 {
        // We can do a simple byte copy.
        asm!(
            "ldrb {2},[{0}]",
            "strb {2},[{1}]",
            in(reg) src, in(reg) dst, out(reg) _,
        );
    } else {
        // When we don't have an optimized path, we just disable IRQs.
        crate::interrupt::free(|_| ptr::write_volatile(dst, ptr::read_volatile(src)));
    }
}

#[allow(unused_assignments)]
unsafe fn transfer_align4_thumb<T: Copy>(mut dst: *mut T, mut src: *const T) {
    let size = mem::size_of::<T>();

    if size <= 4 {
        // We use assembly here regardless to just do the word aligned copy. This
        // ensures it's done with a single ldr/str instruction.
        asm!(
            "ldr {2},[{0}]",
            "str {2},[{1}]",
            inout(reg) src, in(reg) dst, out(reg) _,
        );
    } else if size <= 8 {
        // Starting at size == 8, we begin using ldmia/stmia to load/save multiple
        // words in one instruction, avoiding IRQs from interrupting our operation.
        asm!(
            "ldmia {0}!, {{r2-r3}}",
            "stmia {1}!, {{r2-r3}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _,
        );
    } else if size <= 12 {
        asm!(
            "ldmia {0}!, {{r2-r4}}",
            "stmia {1}!, {{r2-r4}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _,
        );
    } else if size <= 16 {
        asm!(
            "ldmia {0}!, {{r2-r5}}",
            "stmia {1}!, {{r2-r5}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _, out("r5") _,
        );
    } else {
        unimplemented!("This should be done via transfer_arm.");
    }
}

#[instruction_set(arm::a32)]
#[allow(unused_assignments)]
unsafe fn transfer_align4_arm<T: Copy>(mut dst: *mut T, mut src: *const T) {
    let size = mem::size_of::<T>();

    if size <= 16 {
        unimplemented!("This should be done via transfer_thumb.");
    } else if size <= 20 {
        // Starting at size == 16, we have to switch to ARM due to lack of
        // accessible registers in THUMB mode.
        asm!(
            "ldmia {0}!, {{r2-r5,r7}}",
            "stmia {1}!, {{r2-r5,r7}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r7") _,
        );
    } else if size <= 24 {
        asm!(
            "ldmia {0}!, {{r2-r5,r7-r8}}",
            "stmia {1}!, {{r2-r5,r7-r8}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r7") _,
            out("r8") _,
        );
    } else if size <= 28 {
        asm!(
            "ldmia {0}!, {{r2-r5,r7-r9}}",
            "stmia {1}!, {{r2-r5,r7-r9}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r7") _,
            out("r8") _, out("r9") _,
        );
    } else if size <= 32 {
        asm!(
            "ldmia {0}!, {{r2-r5,r7-r10}}",
            "stmia {1}!, {{r2-r5,r7-r10}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r7") _,
            out("r8") _, out("r9") _, out("r10") _,
        );
    } else if size <= 36 {
        asm!(
            "ldmia {0}!, {{r2-r5,r7-r10,r12}}",
            "stmia {1}!, {{r2-r5,r7-r10,r12}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r7") _,
            out("r8") _, out("r9") _, out("r10") _, out("r12") _,
        );
    } else if size <= 40 {
        asm!(
            "ldmia {0}!, {{r2-r5,r7-r10,r12,r14}}",
            "stmia {1}!, {{r2-r5,r7-r10,r12,r14}}",
            inout(reg) src, inout(reg) dst,
            out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r7") _,
            out("r8") _, out("r9") _, out("r10") _, out("r12") _, out("r14") _,
        );
    } else {
        // r13 is sp, and r15 is pc. Neither are usable
        unimplemented!("Copy too large for use of ldmia/stmia.");
    }
}

/// The internal function for swapping the current value of a [`Static`] with
/// another value.
unsafe fn exchange<T>(dst: *mut T, src: *const T) -> T {
    let align = mem::align_of::<T>();
    let size = mem::size_of::<T>();
    if size == 0 {
        // Do nothing with ZSTs.
        ptr::read(dst)
    } else if size <= 4 && align % 4 == 0 {
        // Swap a single word with the SWP instruction.
        let val = ptr::read(src as *const u32);
        let new_val = exchange_align4_arm(dst, val);
        ptr::read(&new_val as *const _ as *const T)
    } else if size == 1 {
        // Swap a byte with the SWPB instruction.
        let val = ptr::read(src as *const u8);
        let new_val = exchange_align1_arm(dst, val);
        ptr::read(&new_val as *const _ as *const T)
    } else {
        // fallback
        crate::interrupt::free(|_| {
            let cur = ptr::read_volatile(dst);
            ptr::write_volatile(dst, ptr::read_volatile(src));
            cur
        })
    }
}

#[instruction_set(arm::a32)]
unsafe fn exchange_align4_arm<T>(dst: *mut T, i: u32) -> u32 {
    let out;
    asm!("swp {2}, {1}, [{0}]", in(reg) dst, in(reg) i, lateout(reg) out);
    out
}

#[instruction_set(arm::a32)]
unsafe fn exchange_align1_arm<T>(dst: *mut T, i: u8) -> u8 {
    let out;
    asm!("swpb {2}, {1}, [{0}]", in(reg) dst, in(reg) i, lateout(reg) out);
    out
}

/// A helper that implements static variables.
///
/// It ensures that even if you use the same static variable in both an IRQ
/// and normal code, the IRQ will never observe an invalid value of the
/// variable.
///
/// This type only works with owned values. If you need to work with borrows,
/// consider using [`sync::Mutex`](`crate::sync::Mutex`) instead.
///
/// ## Performance
///
/// Writing or reading from a static variable is efficient under the following
/// conditions:
///
/// * The type is aligned to 4 bytes and can be stored in 40 bytes or less.
/// * The type is aligned to 2 bytes and can be stored in 2 bytes.
/// * The type is can be stored in a single byte.
///
/// Replacing the current value of the static variable is efficient under the
/// following conditions:
///
/// * The type is aligned to 4 bytes and can be stored in 4 bytes or less.
/// * The type is can be stored in a single byte.
///
/// When these conditions are not met, static variables are handled using a
/// fallback routine that disables IRQs and does a normal copy. This can be
/// dangerous as disabling IRQs can cause your program to miss out on important
/// interrupts such as V-Blank.
///
/// Consider using [`sync::Mutex`](`crate::sync::Mutex`) instead if you need to
/// use a large amount of operations that would cause IRQs to be disabled. Also
/// consider using `#[repr(align(4))]` to force proper alignment for your type.
pub struct Static<T> {
    data: UnsafeCell<T>,
}
impl<T> Static<T> {
    /// Creates a new static variable.
    pub const fn new(val: T) -> Self {
        Static { data: UnsafeCell::new(val) }
    }

    /// Replaces the current value of the static variable with another, and
    /// returns the old value.
    #[allow(clippy::needless_pass_by_value)] // critical for safety
    pub fn replace(&self, val: T) -> T {
        unsafe { exchange(self.data.get(), &val) }
    }

    /// Extracts the interior value of the static variable.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}
impl<T: Copy> Static<T> {
    /// Writes a new value into this static variable.
    pub fn write(&self, val: T) {
        unsafe { transfer(self.data.get(), &val) }
    }

    /// Reads a value from this static variable.
    pub fn read(&self) -> T {
        unsafe {
            let mut out: mem::MaybeUninit<T> = mem::MaybeUninit::uninit();
            transfer(out.as_mut_ptr(), self.data.get());
            out.assume_init()
        }
    }
}
impl<T: Default> Default for Static<T> {
    fn default() -> Self {
        Static::new(T::default())
    }
}
unsafe impl<T> Send for Static<T> {}
unsafe impl<T> Sync for Static<T> {}

#[cfg(test)]
mod test {
    use crate::Gba;
    use crate::interrupt::Interrupt;
    use crate::sync::Static;
    use crate::timer::Divider;

    fn write_read_concurrency_test_impl<const COUNT: usize>(gba: &mut Gba) {
        let sentinel = [0x12345678; COUNT];
        let value: Static<[u32; COUNT]> = Static::new(sentinel);

        // set up a timer and an interrupt that uses the timer
        let mut timer = gba.timers.timers().timer2;
        timer.set_cascade(false);
        timer.set_divider(Divider::Divider1);
        timer.set_overflow_amount(1049);
        timer.set_interrupt(true);
        timer.set_enabled(true);

        let _int = crate::interrupt::add_interrupt_handler(Interrupt::Timer2, |_| {
            value.write(sentinel);
        });

        // the actual main test loop
        let mut interrupt_seen = false;
        let mut no_interrupt_seen = false;
        for i in 0..250000 {
            // write to the static
            let new_value = [i; COUNT];
            value.write(new_value);

            // check the current value
            let current = value.read();
            if current == new_value {
                no_interrupt_seen = true;
            } else if current == sentinel {
                interrupt_seen = true;
            } else {
                panic!("Unexpected value found in `Static`.");
            }

            // we return as soon as we've seen both the value written by the main thread
            // and interrupt
            if interrupt_seen && no_interrupt_seen {
                timer.set_enabled(false);
                return
            }

            if i % 8192 == 0 && i != 0 {
                timer.set_overflow_amount(1049 + (i / 64) as u16);
            }
        }
        panic!("Concurrency test timed out: {}", COUNT)
    }

    #[test_case]
    fn write_read_concurrency_test(gba: &mut Gba) {
        write_read_concurrency_test_impl::<1>(gba);
        write_read_concurrency_test_impl::<2>(gba);
        write_read_concurrency_test_impl::<3>(gba);
        write_read_concurrency_test_impl::<4>(gba);
        write_read_concurrency_test_impl::<5>(gba);
        write_read_concurrency_test_impl::<6>(gba);
        write_read_concurrency_test_impl::<7>(gba);
        write_read_concurrency_test_impl::<8>(gba);
        write_read_concurrency_test_impl::<9>(gba);
        write_read_concurrency_test_impl::<10>(gba);
    }
}