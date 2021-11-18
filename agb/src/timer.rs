use crate::memory_mapped::MemoryMapped;

const fn timer_data(timer: usize) -> MemoryMapped<u16> {
    unsafe { MemoryMapped::new(0x0400_0100 + 4 * timer) }
}

const fn timer_control(timer: usize) -> MemoryMapped<u16> {
    unsafe { MemoryMapped::new(0x0400_0102 + 4 * timer) }
}

#[derive(Clone, Copy)]
pub enum Timer {
    Timer0,
    Timer1,
    Timer2,
    Timer3,
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
    fn get_as_bits(&self) -> u16 {
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
pub struct TimerController {}

impl TimerController {
    pub(crate) const unsafe fn new() -> Self {
        Self {}
    }

    pub fn set_overflow_amount(&mut self, timer: Timer, n: u16) {
        timer.set_overflow_amount(n);
    }

    pub fn get_value(&mut self, timer: Timer) -> u16 {
        timer.get_value()
    }

    pub fn set_divider(&mut self, timer: Timer, divider: Divider) {
        timer
            .control_register()
            .set_bits(divider.get_as_bits(), 2, 0);
    }

    pub fn set_enabled(&mut self, timer: Timer, enabled: bool) {
        let bit = if enabled { 1 } else { 0 };
        timer.control_register().set_bits(bit, 1, 7);
    }

    pub fn set_cascade(&mut self, timer: Timer, cascade: bool) {
        let bit = if cascade { 1 } else { 0 };
        timer.control_register().set_bits(bit, 1, 2);
    }
}

impl Timer {
    fn set_overflow_amount(&self, n: u16) {
        let count_up_value = 0u16.wrapping_sub(n);
        self.data_register().set(count_up_value);
    }

    fn get_value(&self) -> u16 {
        self.data_register().get()
    }

    fn data_register(&self) -> MemoryMapped<u16> {
        timer_data(self.get_timer_number())
    }

    fn control_register(&self) -> MemoryMapped<u16> {
        timer_control(self.get_timer_number())
    }

    fn get_timer_number(&self) -> usize {
        use Timer::*;

        match self {
            Timer0 => 0,
            Timer1 => 1,
            Timer2 => 2,
            Timer3 => 3,
        }
    }
}
