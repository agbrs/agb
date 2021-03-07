use crate::memory_mapped::MemoryMapped;

pub enum Interrupt {
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
