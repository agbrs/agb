use crate::memory_mapped::MemoryMapped;

pub(crate) enum Interrupt {
    VBlank,
    HBlank,
    VCounter,
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    Serial,
    Dma0,
    Dma1,
    Dma2,
    Dma3,
    Keypad,
    Gamepak,
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
