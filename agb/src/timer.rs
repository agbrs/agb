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
pub struct Timer<const N: usize> {}

#[non_exhaustive]
pub struct TimerController {
    pub timer0: Timer<0>,
    pub timer1: Timer<1>,
    pub timer2: Timer<2>,
    pub timer3: Timer<3>,
}

impl TimerController {
    pub(crate) const unsafe fn new() -> Self {
        Self {
            timer0: Timer::new(),
            timer1: Timer::new(),
            timer2: Timer::new(),
            timer3: Timer::new(),
        }
    }
}

impl<const N: usize> Timer<N> {
    const unsafe fn new() -> Self {
        Self {}
    }

    pub fn set_overflow_amount(&mut self, n: u16) {
        let count_up_value = 0u16.wrapping_sub(n);
        self.data_register().set(count_up_value);
    }

    pub fn get_value(&self) -> u16 {
        self.data_register().get()
    }

    pub fn set_divider(&mut self, divider: Divider) {
        self.control_register()
            .set_bits(divider.get_as_bits(), 2, 0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        let bit = if enabled { 1 } else { 0 };
        self.control_register().set_bits(bit, 1, 7);
    }

    pub fn set_cascade(&mut self, cascade: bool) {
        let bit = if cascade { 1 } else { 0 };
        self.control_register().set_bits(bit, 1, 2);
    }

    fn data_register(&self) -> MemoryMapped<u16> {
        timer_data(self.get_timer_number())
    }

    fn control_register(&self) -> MemoryMapped<u16> {
        timer_control(self.get_timer_number())
    }

    fn get_timer_number(&self) -> usize {
        N
    }
}
