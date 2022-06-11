use crate::memory_mapped::MemoryMapped;

const fn timer_data(timer: usize) -> MemoryMapped<u16> {
    unsafe { MemoryMapped::new(0x0400_0100 + 4 * timer) }
}

const fn timer_control(timer: usize) -> MemoryMapped<u16> {
    unsafe { MemoryMapped::new(0x0400_0102 + 4 * timer) }
}

#[derive(Clone, Copy)]
pub enum Divider {
    // 16.78MHz or 59.59ns
    Divider1,
    // 262.21kHz or 3.815us
    Divider64,
    // 65.536kHz or 15.26us
    Divider256,
    // 16.384kHz or 61.04us
    Divider1024,
}

impl Divider {
    fn as_bits(&self) -> u16 {
        use Divider::*;

        match self {
            Divider1 => 0,
            Divider64 => 1,
            Divider256 => 2,
            Divider1024 => 3,
        }
    }
}

#[non_exhaustive]
pub struct Timer {
    timer_number: u16,
}

#[non_exhaustive]
pub struct Timers {
    pub timer2: Timer,
    pub timer3: Timer,
}

impl Timers {
    pub(crate) unsafe fn new() -> Self {
        Self {
            timer2: Timer::new(2),
            timer3: Timer::new(3),
        }
    }
}

impl Timer {
    pub(crate) unsafe fn new(timer_number: u16) -> Self {
        let new_timer = Self { timer_number };
        new_timer.data_register().set(0);
        new_timer.control_register().set(0);

        new_timer
    }

    pub fn set_overflow_amount(&mut self, n: u16) {
        let count_up_value = 0u16.wrapping_sub(n);
        self.data_register().set(count_up_value);
    }

    pub fn value(&self) -> u16 {
        self.data_register().get()
    }

    pub fn set_divider(&mut self, divider: Divider) {
        self.control_register().set_bits(divider.as_bits(), 2, 0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        let bit = if enabled { 1 } else { 0 };
        self.control_register().set_bits(bit, 1, 7);
    }

    pub fn set_cascade(&mut self, cascade: bool) {
        let bit = if cascade { 1 } else { 0 };
        self.control_register().set_bits(bit, 1, 2);
    }

    pub fn set_interrupt(&mut self, interrupt: bool) {
        let bit = interrupt as u16;
        self.control_register().set_bits(bit, 1, 6);
    }

    fn data_register(&self) -> MemoryMapped<u16> {
        timer_data(self.timer_number())
    }

    fn control_register(&self) -> MemoryMapped<u16> {
        timer_control(self.timer_number())
    }

    fn timer_number(&self) -> usize {
        self.timer_number as usize
    }

    pub fn interrupt(&self) -> crate::interrupt::Interrupt {
        use crate::interrupt::Interrupt;
        match self.timer_number {
            0 => Interrupt::Timer0,
            1 => Interrupt::Timer1,
            2 => Interrupt::Timer2,
            3 => Interrupt::Timer3,
            _ => unreachable!(),
        }
    }
}

#[non_exhaustive]
pub struct TimerController {}

impl TimerController {
    pub(crate) const fn new() -> Self {
        Self {}
    }

    pub fn timers(&mut self) -> Timers {
        unsafe { Timers::new() }
    }
}
