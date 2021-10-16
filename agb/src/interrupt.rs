use core::{
    cell::{Cell, RefCell, RefMut},
    marker::{PhantomData, PhantomPinned},
    ops::{Deref, DerefMut},
    pin::Pin,
};

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

extern "C" {
    fn InterruptSwap(a: u32) -> u32;
}

struct Disable {
    previous: u32,
}

impl Drop for Disable {
    fn drop(&mut self) {
        unsafe { InterruptSwap(self.previous) };
    }
}

fn temporary_interrupt_disable() -> Disable {
    Disable {
        previous: unsafe { InterruptSwap(0) },
    }
}

struct InterruptRoot {
    next: Cell<*const InterruptClosure>,
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
    for (i, root) in unsafe { &INTERRUPT_TABLE }.iter().enumerate() {
        if (1 << i) & interrupt != 0 {
            root.trigger_interrupts();
        }
    }

    interrupt
}

pub struct InterruptClosureBounded<'a> {
    c: InterruptClosure,
    _phantom: PhantomData<&'a ()>,
    _unpin: PhantomPinned,
}

struct InterruptClosure {
    closure: *const (dyn Fn(Key)),
    next: Cell<*const InterruptClosure>,
    root: *const InterruptRoot,
}

impl InterruptRoot {
    fn trigger_interrupts(&self) {
        let mut c = self.next.get();
        while !c.is_null() {
            let closure_ptr = unsafe { &*c }.closure;
            let closure_ref = unsafe { &*closure_ptr };
            closure_ref(Key());
            c = unsafe { &*c }.next.get();
        }
    }
}

impl Drop for InterruptClosure {
    fn drop(&mut self) {
        let root = unsafe { &*self.root };
        root.reduce();
        let mut c = root.next.get();
        let own_pointer = self as *const _;
        if c == own_pointer {
            unsafe { &*self.root }.next.set(self.next.get());
            return;
        }
        loop {
            let p = unsafe { &*c }.next.get();
            if p == own_pointer {
                unsafe { &*c }.next.set(self.next.get());
                return;
            }
            c = p;
        }
    }
}

fn interrupt_to_root(interrupt: Interrupt) -> &'static InterruptRoot {
    unsafe { &INTERRUPT_TABLE[interrupt as usize] }
}

fn get_interrupt_handle_root<'a>(
    f: &'a dyn Fn(Key),
    root: &InterruptRoot,
) -> InterruptClosureBounded<'a> {
    InterruptClosureBounded {
        c: InterruptClosure {
            closure: unsafe { core::mem::transmute(f as *const _) },
            next: Cell::new(core::ptr::null()),
            root: root as *const _,
        },
        _phantom: PhantomData,
        _unpin: PhantomPinned,
    }
}

/// The [add_interrupt_handler!] macro should be used instead of this function.
/// Creates an interrupt handler from a closure.
pub fn get_interrupt_handle(
    f: &(dyn Fn(Key) + Send + Sync),
    interrupt: Interrupt,
) -> InterruptClosureBounded {
    let root = interrupt_to_root(interrupt);

    get_interrupt_handle_root(f, root)
}

/// The [add_interrupt_handler!] macro should be used instead of this.
/// Adds an interrupt handler to the interrupt table such that when that
/// interrupt is triggered the closure is called.
pub fn add_interrupt<'a>(interrupt: Pin<&'a InterruptClosureBounded<'a>>) {
    let root = unsafe { &*interrupt.c.root };
    root.add();
    let mut c = root.next.get();
    if c.is_null() {
        root.next.set((&interrupt.c) as *const _);
        return;
    }
    loop {
        let p = unsafe { &*c }.next.get();
        if p.is_null() {
            unsafe { &*c }.next.set((&interrupt.c) as *const _);
            return;
        }

        c = p;
    }
}

#[macro_export]
/// Creates a new interrupt handler in the current scope, when this scope drops
/// the interrupt handler is removed. Note that this returns nothing, but some
/// stack space is used. The interrupt handler is of the form `Fn(Key) + Send +
/// Sync` where Key can be used to unlock a mutex without checking whether
/// interrupts need to be disabled, as during an interrupt interrupts are
/// disabled.
///
/// # Usage
/// ```
/// add_interrupt_handler!(Interrupt::VBlank, |key| agb::println!("hello world!"));
/// ```
///
macro_rules! add_interrupt_handler {
    ($interrupt: expr, $handler: expr) => {
        let a = $handler;
        let a = $crate::interrupt::get_interrupt_handle(&a, $interrupt);
        let a = unsafe { core::pin::Pin::new_unchecked(&a) };
        $crate::interrupt::add_interrupt(a);
    };
}

#[test_case]
fn test_vblank_interrupt_handler(_gba: &mut crate::Gba) {
    {
        let counter = Mutex::new(0);
        let counter_2 = Mutex::new(0);
        add_interrupt_handler!(Interrupt::VBlank, |key| *counter.lock_with_key(&key) += 1);
        add_interrupt_handler!(Interrupt::VBlank, |_| *counter_2.lock() += 1);

        let vblank = VBlank::get();

        while *counter.lock() < 100 || *counter_2.lock() < 100 {
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

pub struct Mutex<T> {
    internal: RefCell<T>,
}

pub struct MutexRef<'a, T> {
    r: RefMut<'a, T>,
    _d: Disable,
}

#[non_exhaustive]
pub struct Key();

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn lock(&self) -> MutexRef<T> {
        let _a = temporary_interrupt_disable();
        MutexRef {
            r: self.internal.borrow_mut(),
            _d: _a,
        }
    }

    pub fn lock_with_key(&self, _key: &Key) -> RefMut<T> {
        self.internal.borrow_mut()
    }

    pub const fn new(val: T) -> Self {
        Mutex {
            internal: RefCell::new(val),
        }
    }
}

impl<'a, T> Deref for MutexRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.r
    }
}

impl<'a, T> DerefMut for MutexRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.r
    }
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
