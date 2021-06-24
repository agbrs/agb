use core::{
    cell::{Cell, UnsafeCell},
    marker::{PhantomData, PhantomPinned},
    ops::{Deref, DerefMut},
    pin::Pin,
};

use crate::memory_mapped::MemoryMapped;

#[allow(dead_code)]
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

const ENABLED_INTERRUPTS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x04000200) };
const INTERRUPTS_ENABLED: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x04000208) };

pub(crate) fn enable(interrupt: Interrupt) {
    let _interrupt_token = temporary_interrupt_disable();
    let interrupt = interrupt as usize;
    let enabled = ENABLED_INTERRUPTS.get() | (1 << (interrupt as u16));
    ENABLED_INTERRUPTS.set(enabled);
}

pub(crate) fn disable(interrupt: Interrupt) {
    let _interrupt_token = temporary_interrupt_disable();
    let interrupt = interrupt as usize;
    let enabled = ENABLED_INTERRUPTS.get() & !(1 << (interrupt as u16));
    ENABLED_INTERRUPTS.set(enabled);
}

pub(crate) struct Disable {}

impl Drop for Disable {
    fn drop(&mut self) {
        enable_interrupts();
    }
}

pub(crate) fn temporary_interrupt_disable() -> Disable {
    disable_interrupts();
    Disable {}
}

pub(crate) fn enable_interrupts() {
    INTERRUPTS_ENABLED.set(1);
}

pub(crate) fn disable_interrupts() {
    INTERRUPTS_ENABLED.set(0);
}

pub(crate) struct InterruptRoot {
    next: Cell<*const InterruptClosure>,
}

impl InterruptRoot {
    const fn new() -> Self {
        InterruptRoot {
            next: Cell::new(core::ptr::null()),
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
pub extern "C" fn __RUST_INTERRUPT_HANDLER(interrupt: u16) {
    for i in 0..=13_u16 {
        if (1 << i) & interrupt != 0 {
            let root = unsafe { &INTERRUPT_TABLE[i as usize] };
            root.trigger_interrupts();
        }
    }
}

pub struct InterruptClosureBounded<'a> {
    c: InterruptClosure,
    _phantom: PhantomData<&'a ()>,
    _unpin: PhantomPinned,
}

struct InterruptClosure {
    closure: *const (dyn Fn()),
    next: Cell<*const InterruptClosure>,
    root: *const InterruptRoot,
}

impl InterruptRoot {
    fn trigger_interrupts(&self) {
        let mut c = self.next.get();
        while !c.is_null() {
            let closure_ptr = unsafe { &*c }.closure;
            let closure_ref = unsafe { &*closure_ptr };
            closure_ref();
            c = unsafe { &*c }.next.get();
        }
    }
}

impl Drop for InterruptClosure {
    fn drop(&mut self) {
        let mut c = unsafe { &*self.root }.next.get();
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
    f: &'a dyn Fn(),
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

pub fn get_interrupt_handle(
    f: &(dyn Fn() + Send + Sync),
    interrupt: Interrupt,
) -> InterruptClosureBounded {
    let root = interrupt_to_root(interrupt);

    get_interrupt_handle_root(f, root)
}

pub fn add_interrupt<'a>(interrupt: Pin<&'a InterruptClosureBounded<'a>>) {
    let root = unsafe { &*interrupt.c.root };
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

#[test_case]
fn test_vblank_interrupt_handler(gba: &mut crate::Gba) {
    {
        let counter = Mutex::new(0);
        let counter_2 = Mutex::new(0);

        let mut vblank_interrupt = || *counter.lock() += 1;
        let mut vblank_interrupt_2 = || *counter_2.lock() += 1;

        let interrupt_closure = get_interrupt_handle(&mut vblank_interrupt, Interrupt::VBlank);
        let interrupt_closure = unsafe { Pin::new_unchecked(&interrupt_closure) };
        add_interrupt(interrupt_closure);

        let interrupt_closure_2 = get_interrupt_handle(&mut vblank_interrupt_2, Interrupt::VBlank);
        let interrupt_closure_2 = unsafe { Pin::new_unchecked(&interrupt_closure_2) };
        add_interrupt(interrupt_closure_2);

        let vblank = gba.display.vblank.get();

        while *counter.lock() < 100 || *counter_2.lock() < 100 {
            vblank.wait_for_VBlank();
        }
    }

    assert_eq!(
        interrupt_to_root(Interrupt::VBlank).next.get(),
        core::ptr::null(),
        "expected the interrupt table for vblank to be empty"
    );
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MutexState {
    Unlocked,
    Locked(bool),
}

pub struct Mutex<T> {
    internal: UnsafeCell<T>,
    state: UnsafeCell<MutexState>,
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn lock(&self) -> MutexRef<T> {
        let state = INTERRUPTS_ENABLED.get();
        INTERRUPTS_ENABLED.set(0);
        assert_eq!(
            unsafe { *self.state.get() },
            MutexState::Unlocked,
            "mutex must be unlocked to be able to lock it"
        );
        unsafe { *self.state.get() = MutexState::Locked(state != 0) };
        MutexRef {
            internal: &self.internal,
            state: &self.state,
        }
    }
    pub fn new(val: T) -> Self {
        Mutex {
            internal: UnsafeCell::new(val),
            state: UnsafeCell::new(MutexState::Unlocked),
        }
    }
}

pub struct MutexRef<'a, T> {
    internal: &'a UnsafeCell<T>,
    state: &'a UnsafeCell<MutexState>,
}

impl<'a, T> Drop for MutexRef<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let state = &mut *self.state.get();
            let prev_state = *state;
            *state = MutexState::Unlocked;
            match prev_state {
                MutexState::Locked(b) => INTERRUPTS_ENABLED.set(b as u16),
                MutexState::Unlocked => {}
            }
        }
    }
}

impl<'a, T> Deref for MutexRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.internal.get() }
    }
}

impl<'a, T> DerefMut for MutexRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.internal.get() }
    }
}
