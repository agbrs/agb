use core::cell::Cell;

use alloc::{rc::Rc, vec::Vec};
use bare_metal::CriticalSection;

use crate::{display::DISPLAY_STATUS, memory_mapped::MemoryMapped, sync::Static};

#[derive(Clone, Copy)]
pub enum Interrupt {
    VBlank = 0,
    HBlank = 1,
    VCounter = 2,
    Timer0 = 3,
    Timer1 = 4,
    Timer2 = 5,
    Timer3 = 6,
    Serial = 7,
    Dma0 = 8,
    Dma1 = 9,
    Dma2 = 10,
    Dma3 = 11,
    Keypad = 12,
    Gamepak = 13,
}

impl Interrupt {
    fn enable(self, _cs: CriticalSection) {
        self.other_things_to_enable_interrupt();
        let interrupt = self as usize;
        let enabled = ENABLED_INTERRUPTS.get() | (1 << (interrupt as u16));
        ENABLED_INTERRUPTS.set(enabled);
    }

    fn disable(self, _cs: CriticalSection) {
        self.other_things_to_disable_interrupt();
        let interrupt = self as usize;
        let enabled = ENABLED_INTERRUPTS.get() & !(1 << (interrupt as u16));
        ENABLED_INTERRUPTS.set(enabled);
    }

    fn other_things_to_enable_interrupt(self) {
        match self {
            Interrupt::VBlank => {
                DISPLAY_STATUS.set_bits(1, 1, 3);
            }
            Interrupt::HBlank => {
                DISPLAY_STATUS.set_bits(1, 1, 4);
            }
            _ => {}
        }
    }

    fn other_things_to_disable_interrupt(self) {
        match self {
            Interrupt::VBlank => {
                DISPLAY_STATUS.set_bits(0, 1, 3);
            }
            Interrupt::HBlank => {
                DISPLAY_STATUS.set_bits(0, 1, 4);
            }
            _ => {}
        }
    }
}

const ENABLED_INTERRUPTS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x04000200) };
const INTERRUPTS_ENABLED: MemoryMapped<u32> = unsafe { MemoryMapped::new(0x04000208) };

extern "C" {
    static mut __INTERRUPT_NEST: u32;
}

struct InterruptRoot {
    interrupts: Vec<Rc<dyn Fn() + Send + Sync>>,
}

impl InterruptRoot {
    const fn new() -> Self {
        InterruptRoot {
            interrupts: Vec::new(),
        }
    }
}

static mut INTERRUPT_TABLE: [InterruptRoot; 14] = [
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
    InterruptRoot::new(),
];

#[no_mangle]
extern "C" fn __RUST_INTERRUPT_HANDLER(interrupt: u16) -> u16 {
    for (i, root) in unsafe { INTERRUPT_TABLE.iter().enumerate() } {
        if (1 << i) & interrupt != 0 {
            root.trigger_interrupts();
        }
    }

    interrupt
}

pub struct InterruptHandler {
    kind: Interrupt,
    closure: Rc<dyn Fn() + Send + Sync + 'static>,
}

impl Drop for InterruptHandler {
    fn drop(&mut self) {
        free(|cs| {
            let root = unsafe { interrupt_to_root(self.kind) };
            root.interrupts.retain(|x| {
                !core::ptr::eq::<dyn Fn() + Send + Sync + 'static>(&**x, &*self.closure)
            });
            if root.interrupts.is_empty() {
                self.kind.disable(cs);
            }
        });
    }
}

impl InterruptRoot {
    fn trigger_interrupts(&self) {
        for interrupt in self.interrupts.iter() {
            (interrupt)();
        }
    }
}

unsafe fn interrupt_to_root(interrupt: Interrupt) -> &'static mut InterruptRoot {
    unsafe { &mut INTERRUPT_TABLE[interrupt as usize] }
}

#[must_use]
/// Adds an interrupt handler as long as the returned value is alive.
///
/// # Safety
/// * You *must not* allocate in an interrupt.
///     - Many functions in agb allocate and it isn't always clear.
///
/// # Staticness
/// * The closure must be static because forgetting the interrupt handler would
///   cause a use after free.
///
/// # Examples
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// # fn foo() {
/// use agb::interrupt::{add_interrupt_handler, Interrupt};
/// // Safety: doesn't allocate
/// let _a = unsafe {
///     add_interrupt_handler(Interrupt::VBlank, || {
///         agb::println!("Woah there! There's been a vblank!");
///     })
/// };
/// # }
/// ```
pub unsafe fn add_interrupt_handler(
    interrupt: Interrupt,
    handler: impl Fn() + Send + Sync + 'static,
) -> InterruptHandler {
    fn inner(
        interrupt: Interrupt,
        handle: Rc<dyn Fn() + Send + Sync + 'static>,
        cs: CriticalSection,
    ) -> InterruptHandler {
        let interrupts = unsafe { interrupt_to_root(interrupt) };

        if interrupts.interrupts.is_empty() {
            interrupt.enable(cs);
        }

        interrupts.interrupts.push(handle.clone());

        InterruptHandler {
            kind: interrupt,
            closure: handle,
        }
    }

    free(|cs| inner(interrupt, Rc::new(handler), cs))
}

/// How you can access mutexes outside of interrupts by being given a
/// [`CriticalSection`]
///
/// [`CriticalSection`]: bare_metal::CriticalSection
pub fn free<F, R>(mut f: F) -> R
where
    F: FnOnce(CriticalSection) -> R,
{
    let enabled = INTERRUPTS_ENABLED.get();

    INTERRUPTS_ENABLED.set(0);

    // prevents the contents of the function from being reordered before IME is disabled.
    crate::sync::memory_write_hint(&mut f);

    let mut r = f(unsafe { CriticalSection::new() });

    // prevents the contents of the function from being reordered after IME is re-enabled.
    crate::sync::memory_write_hint(&mut r);

    INTERRUPTS_ENABLED.set(enabled);
    r
}

static NUM_VBLANKS: Static<usize> = Static::new(0); // overflows after 2.27 years
static HAS_CREATED_INTERRUPT: Static<bool> = Static::new(false);

#[non_exhaustive]
pub struct VBlank {
    last_waited_number: Cell<usize>,
}

impl VBlank {
    /// Handles setting up everything required to be able to use the wait for
    /// interrupt syscall.
    #[must_use]
    pub fn get() -> Self {
        if !HAS_CREATED_INTERRUPT.read() {
            // safety: we don't allocate in the interrupt
            let handler = unsafe {
                add_interrupt_handler(Interrupt::VBlank, || {
                    NUM_VBLANKS.write(NUM_VBLANKS.read() + 1);
                })
            };
            core::mem::forget(handler);

            HAS_CREATED_INTERRUPT.write(true);
        }

        VBlank {
            last_waited_number: Cell::new(NUM_VBLANKS.read()),
        }
    }
    /// Pauses CPU until vblank interrupt is triggered where code execution is
    /// resumed.
    pub fn wait_for_vblank(&self) {
        let last_waited_number = self.last_waited_number.get();
        self.last_waited_number.set(NUM_VBLANKS.read() + 1);

        if last_waited_number < NUM_VBLANKS.read() {
            return;
        }

        crate::syscall::wait_for_vblank();
    }
}

#[must_use]
/// The behaviour of this function is undefined in the sense that it will output
/// some information in some way that can be interpreted in a way to give some
/// profiling information. What it outputs, how it outputs it, and how to
/// interpret it are all subject to change at any time.
///
/// With that out of the way, the current version will, in mgba, output the
/// program counter at regular intervals. This can be used to see hot functions
/// using, for example, addr2line.
pub fn profiler(timer: &mut crate::timer::Timer, period: u16) -> InterruptHandler {
    timer.set_interrupt(true);
    timer.set_overflow_amount(period);
    timer.set_enabled(true);

    unsafe {
        add_interrupt_handler(timer.interrupt(), || {
            crate::println!("{:#010x}", crate::program_counter_before_interrupt());
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_interrupt_table_length(_gba: &mut crate::Gba) {
        assert_eq!(
            unsafe { INTERRUPT_TABLE.len() },
            Interrupt::Gamepak as usize + 1,
            "interrupt table should be able to store gamepak interrupt"
        );
    }
}
