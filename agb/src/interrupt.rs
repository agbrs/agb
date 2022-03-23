use core::{
    cell::Cell,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
};

use alloc::boxed::Box;
use bare_metal::CriticalSection;

use crate::{display::DISPLAY_STATUS, memory_mapped::MemoryMapped};

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
    fn enable(self) {
        let _interrupt_token = temporary_interrupt_disable();
        self.other_things_to_enable_interrupt();
        let interrupt = self as usize;
        let enabled = ENABLED_INTERRUPTS.get() | (1 << (interrupt as u16));
        ENABLED_INTERRUPTS.set(enabled);
    }

    fn disable(self) {
        let _interrupt_token = temporary_interrupt_disable();
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
const INTERRUPTS_ENABLED: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x04000208) };

struct Disable {
    pre: u16,
}

impl Drop for Disable {
    fn drop(&mut self) {
        INTERRUPTS_ENABLED.set(self.pre);
    }
}

fn temporary_interrupt_disable() -> Disable {
    let d = Disable {
        pre: INTERRUPTS_ENABLED.get(),
    };
    disable_interrupts();
    d
}

fn disable_interrupts() {
    INTERRUPTS_ENABLED.set(0);
}

struct InterruptRoot {
    next: Cell<*const InterruptInner>,
    count: Cell<i32>,
    interrupt: Interrupt,
}

impl InterruptRoot {
    const fn new(interrupt: Interrupt) -> Self {
        InterruptRoot {
            next: Cell::new(core::ptr::null()),
            count: Cell::new(0),
            interrupt,
        }
    }

    fn reduce(&self) {
        let new_count = self.count.get() - 1;
        if new_count == 0 {
            self.interrupt.disable();
        }
        self.count.set(new_count);
    }

    fn add(&self) {
        let count = self.count.get();
        if count == 0 {
            self.interrupt.enable();
        }
        self.count.set(count + 1);
    }
}

static mut INTERRUPT_TABLE: [InterruptRoot; 14] = [
    InterruptRoot::new(Interrupt::VBlank),
    InterruptRoot::new(Interrupt::HBlank),
    InterruptRoot::new(Interrupt::VCounter),
    InterruptRoot::new(Interrupt::Timer0),
    InterruptRoot::new(Interrupt::Timer1),
    InterruptRoot::new(Interrupt::Timer2),
    InterruptRoot::new(Interrupt::Timer3),
    InterruptRoot::new(Interrupt::Serial),
    InterruptRoot::new(Interrupt::Dma0),
    InterruptRoot::new(Interrupt::Dma1),
    InterruptRoot::new(Interrupt::Dma2),
    InterruptRoot::new(Interrupt::Dma3),
    InterruptRoot::new(Interrupt::Keypad),
    InterruptRoot::new(Interrupt::Gamepak),
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

struct InterruptInner {
    next: Cell<*const InterruptInner>,
    root: *const InterruptRoot,
    closure: *const dyn Fn(&CriticalSection),
    _pin: PhantomPinned,
}

unsafe fn create_interrupt_inner(
    c: impl Fn(&CriticalSection),
    root: *const InterruptRoot,
) -> Pin<Box<InterruptInner>> {
    let c = Box::new(c);
    let c: &dyn Fn(&CriticalSection) = Box::leak(c);
    let c: &dyn Fn(&CriticalSection) = core::mem::transmute(c);
    Box::pin(InterruptInner {
        next: Cell::new(core::ptr::null()),
        root,
        closure: c,
        _pin: PhantomPinned,
    })
}

impl Drop for InterruptInner {
    fn drop(&mut self) {
        inner_drop(unsafe { Pin::new_unchecked(self) });
        fn inner_drop(this: Pin<&mut InterruptInner>) {
            // drop the closure allocation safely
            let _closure_box =
                unsafe { Box::from_raw(this.closure as *mut dyn Fn(&CriticalSection)) };

            // perform the rest of the drop sequence
            let root = unsafe { &*this.root };
            root.reduce();
            let mut c = root.next.get();
            let own_pointer = &*this as *const _;
            if c == own_pointer {
                unsafe { &*this.root }.next.set(this.next.get());
                return;
            }
            loop {
                let p = unsafe { &*c }.next.get();
                if p == own_pointer {
                    unsafe { &*c }.next.set(this.next.get());
                    return;
                }
                c = p;
            }
        }
    }
}

pub struct InterruptHandler<'a> {
    _inner: Pin<Box<InterruptInner>>,
    _lifetime: PhantomData<&'a ()>,
}

impl InterruptRoot {
    fn trigger_interrupts(&self) {
        let mut c = self.next.get();
        while !c.is_null() {
            let closure_ptr = unsafe { &*c }.closure;
            let closure_ref = unsafe { &*closure_ptr };
            closure_ref(unsafe { &CriticalSection::new() });
            c = unsafe { &*c }.next.get();
        }
    }
}

fn interrupt_to_root(interrupt: Interrupt) -> &'static InterruptRoot {
    unsafe { &INTERRUPT_TABLE[interrupt as usize] }
}

#[must_use]
/// Adds an interrupt handler as long as the returned value is alive. The
/// closure takes a [`CriticalSection`] which can be used for mutexes.
///
/// [`CriticalSection`]: bare_metal::CriticalSection
///
/// # Examples
///
/// ```
/// let _a = add_interrupt_handler(Interrupt::VBlank, |_: &CriticalSection| {
///     println!("Woah there! There's been a vblank!");
/// });
/// ```
pub fn add_interrupt_handler<'a>(
    interrupt: Interrupt,
    handler: impl Fn(&CriticalSection) + 'a,
) -> InterruptHandler<'a> {
    fn do_with_inner<'a>(
        interrupt: Interrupt,
        inner: Pin<Box<InterruptInner>>,
    ) -> InterruptHandler<'a> {
        free(|_| {
            let root = interrupt_to_root(interrupt);
            root.add();
            let mut c = root.next.get();
            if c.is_null() {
                root.next.set((&*inner) as *const _);
                return;
            }
            loop {
                let p = unsafe { &*c }.next.get();
                if p.is_null() {
                    unsafe { &*c }.next.set((&*inner) as *const _);
                    return;
                }

                c = p;
            }
        });

        InterruptHandler {
            _inner: inner,
            _lifetime: PhantomData,
        }
    }
    let root = interrupt_to_root(interrupt) as *const _;
    let inner = unsafe { create_interrupt_inner(handler, root) };
    do_with_inner(interrupt, inner)
}

/// How you can access mutexes outside of interrupts by being given a
/// [`CriticalSection`]
///
/// [`CriticalSection`]: bare_metal::CriticalSection
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    let enabled = INTERRUPTS_ENABLED.get();

    disable_interrupts();

    let r = f(unsafe { &CriticalSection::new() });

    INTERRUPTS_ENABLED.set(enabled);
    r
}

#[non_exhaustive]
pub struct VBlank {}

impl VBlank {
    /// Handles setting up everything reqired to be able to use the wait for
    /// interrupt syscall.
    pub fn get() -> Self {
        interrupt_to_root(Interrupt::VBlank).add();
        VBlank {}
    }
    /// Pauses CPU until vblank interrupt is triggered where code execution is
    /// resumed.
    pub fn wait_for_vblank(&self) {
        crate::syscall::wait_for_vblank();
    }
}

impl Drop for VBlank {
    fn drop(&mut self) {
        interrupt_to_root(Interrupt::VBlank).reduce();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bare_metal::Mutex;
    use core::cell::RefCell;

    #[test_case]
    fn test_vblank_interrupt_handler(_gba: &mut crate::Gba) {
        {
            let counter = Mutex::new(RefCell::new(0));
            let counter_2 = Mutex::new(RefCell::new(0));
            let _a = add_interrupt_handler(Interrupt::VBlank, |key: &CriticalSection| {
                *counter.borrow(*key).borrow_mut() += 1
            });
            let _b = add_interrupt_handler(Interrupt::VBlank, |key: &CriticalSection| {
                *counter_2.borrow(*key).borrow_mut() += 1
            });

            let vblank = VBlank::get();

            while free(|key| {
                *counter.borrow(*key).borrow() < 100 || *counter_2.borrow(*key).borrow() < 100
            }) {
                vblank.wait_for_vblank();
            }
        }

        assert_eq!(
            interrupt_to_root(Interrupt::VBlank).next.get(),
            core::ptr::null(),
            "expected the interrupt table for vblank to be empty"
        );
    }

    #[test_case]
    fn test_interrupt_table_length(_gba: &mut crate::Gba) {
        assert_eq!(
            unsafe { INTERRUPT_TABLE.len() },
            Interrupt::Gamepak as usize + 1,
            "interrupt table should be able to store gamepak interrupt"
        );
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

    add_interrupt_handler(timer.interrupt(), |_key: &CriticalSection| {
        crate::println!("{:#010x}", crate::program_counter_before_interrupt());
    })
}
