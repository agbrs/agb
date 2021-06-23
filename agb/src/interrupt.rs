use core::{
    cell::Cell,
    marker::{PhantomData, PhantomPinned},
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

pub fn enable(interrupt: Interrupt) {
    let _interrupt_token = temporary_interrupt_disable();
    let interrupt = interrupt as usize;
    let enabled = ENABLED_INTERRUPTS.get() | (1 << (interrupt as u16));
    ENABLED_INTERRUPTS.set(enabled);
}

pub fn disable(interrupt: Interrupt) {
    let _interrupt_token = temporary_interrupt_disable();
    let interrupt = interrupt as usize;
    let enabled = ENABLED_INTERRUPTS.get() & !(1 << (interrupt as u16));
    ENABLED_INTERRUPTS.set(enabled);
}

pub struct Disable {}

impl Drop for Disable {
    fn drop(&mut self) {
        enable_interrupts();
    }
}

pub fn temporary_interrupt_disable() -> Disable {
    disable_interrupts();
    Disable {}
}

pub fn enable_interrupts() {
    INTERRUPTS_ENABLED.set(1);
}

pub(crate) fn disable_interrupts() {
    INTERRUPTS_ENABLED.set(0);
}

pub struct InterruptRoot {
    next: Cell<*const InterruptClosure>,
}

impl InterruptRoot {
    const fn new() -> Self {
        InterruptRoot {
            next: Cell::new(core::ptr::null()),
        }
    }
}

static mut INTERRUPT_TABLE: Interrupts = Interrupts {
    vblank: InterruptRoot::new(),
    hblank: InterruptRoot::new(),
};

#[no_mangle]
pub extern "C" fn __RUST_INTERRUPT_HANDLER(interrupt: u16) {
    if interrupt & 1 != 0 {
        unsafe { INTERRUPT_TABLE.vblank.trigger_interrupts() };
    };
}

struct Interrupts {
    vblank: InterruptRoot,
    hblank: InterruptRoot,
}

pub struct InterruptClosureBounded<'a> {
    c: InterruptClosure,
    _phantom: PhantomData<&'a ()>,
    _unpin: PhantomPinned,
}

pub struct InterruptClosure {
    closure: *mut (dyn FnMut()),
    next: Cell<*const InterruptClosure>,
    root: *const InterruptRoot,
}

impl InterruptRoot {
    fn trigger_interrupts(&self) {
        let mut count = 0;
        let mut c = self.next.get();
        while !c.is_null() {
            count += 1;
            let closure_ptr = unsafe { &*c }.closure;
            let closure_ref = unsafe { &mut *closure_ptr };
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

fn get_interrupt_handle_root<'a>(
    f: &'a mut dyn FnMut(),
    root: &InterruptRoot,
) -> InterruptClosureBounded<'a> {
    InterruptClosureBounded {
        c: InterruptClosure {
            closure: unsafe { core::mem::transmute(f as *mut _) },
            next: Cell::new(core::ptr::null()),
            root: root as *const _,
        },
        _phantom: PhantomData,
        _unpin: PhantomPinned,
    }
}

pub fn get_interrupt_handle<'a>(
    f: &'a mut dyn FnMut(),
    interrupt: Interrupt,
) -> InterruptClosureBounded<'a> {
    let root = match interrupt {
        Interrupt::VBlank => unsafe { &INTERRUPT_TABLE.vblank },
        _ => unimplemented!(
            "sorry, I haven't yet added this interrupt. Please request it if you need it"
        ),
    };

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
        let mut counter = 0;
        let mut counter_2 = 0;

        let mut vblank_interrupt = || counter += 1;
        let mut vblank_interrupt_2 = || counter_2 += 1;

        let interrupt_closure = get_interrupt_handle(&mut vblank_interrupt, Interrupt::VBlank);
        let interrupt_closure = unsafe { Pin::new_unchecked(&interrupt_closure) };
        add_interrupt(interrupt_closure);

        let interrupt_closure_2 = get_interrupt_handle(&mut vblank_interrupt_2, Interrupt::VBlank);
        let interrupt_closure_2 = unsafe { Pin::new_unchecked(&interrupt_closure_2) };
        add_interrupt(interrupt_closure_2);

        let vblank = gba.display.vblank.get();

        while counter < 100 || counter_2 < 100 {
            vblank.wait_for_VBlank();
        }
    }

    assert_eq!(
        unsafe { INTERRUPT_TABLE.vblank.next.get() },
        core::ptr::null(),
        "expected the interrupt table for vblank to be empty"
    );
}
