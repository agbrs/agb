use core::marker::PhantomData;

use crate::memory_mapped::MemoryMapped;

const fn timer_data(timer: usize) -> MemoryMapped<u16> {
    unsafe { MemoryMapped::new(0x0400_0100 + 4 * timer) }
}

const fn timer_control(timer: usize) -> MemoryMapped<u16> {
    unsafe { MemoryMapped::new(0x0400_0102 + 4 * timer) }
}

#[derive(Clone, Copy)]
/// Divides the cycle precise timer down
///
/// A larger divider will mean the timer is slower. The frequency and cycle
/// durations are given.
pub enum Divider {
    /// 16.78MHz or 59.59ns
    Divider1,
    /// 262.21kHz or 3.815us
    Divider64,
    /// 65.536kHz or 15.26us
    Divider256,
    /// 16.384kHz or 61.04us
    Divider1024,
}

impl Divider {
    fn as_bits(self) -> u16 {
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
/// A 16 bit hardware timer
pub struct Timer {
    timer_number: u16,
}

#[non_exhaustive]
/// A currently hard to use distributor around hardware timers only exposing
/// those that you can freely use in agb.
pub struct Timers<'gba> {
    /// A timer
    pub timer2: Timer,
    /// A timer
    pub timer3: Timer,

    phantom: PhantomData<&'gba ()>,
}

impl Timers<'_> {
    pub(crate) unsafe fn new() -> Self {
        Self {
            timer2: unsafe { Timer::new(2) },
            timer3: unsafe { Timer::new(3) },

            phantom: PhantomData,
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

    /// Causes the timer to overflow at the given value.
    pub fn set_overflow_amount(&mut self, n: u16) -> &mut Self {
        let count_up_value = 0u16.wrapping_sub(n);
        self.data_register().set(count_up_value);
        self
    }

    #[must_use]
    /// The current value of the timer.
    pub fn value(&self) -> u16 {
        self.data_register().get()
    }

    /// Sets the divider of the timer which causes the timer to run at a faster
    /// or slower rate.
    pub fn set_divider(&mut self, divider: Divider) -> &mut Self {
        self.control_register().set_bits(divider.as_bits(), 2, 0);
        self
    }

    /// Sets whether the timer is frozen or enabled.
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        let bit = u16::from(enabled);
        self.control_register().set_bits(bit, 1, 7);
        self
    }

    /// Causes the timer to step when the previous timer overflows. Can be used to create timers that are greater than 16 bit.
    pub fn set_cascade(&mut self, cascade: bool) -> &mut Self {
        let bit = u16::from(cascade);
        self.control_register().set_bits(bit, 1, 2);
        self
    }

    /// Sets whether the relevant interrupt will trigger when this timer overflows
    pub fn set_interrupt(&mut self, interrupt: bool) -> &mut Self {
        let bit = u16::from(interrupt);
        self.control_register().set_bits(bit, 1, 6);
        self
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

    #[must_use]
    /// Gets the relevant interrupt that can directly be used in [`add_interrupt_handler`][crate::interrupt::add_interrupt_handler].
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
/// A distributor of timers that binds the timers lifetime to that of the Gba
/// struct to try ensure unique access to timers.
pub struct TimerController {}

impl TimerController {
    pub(crate) const fn new() -> Self {
        Self {}
    }

    /// Gets the underlying timers.
    pub fn timers(&mut self) -> Timers<'_> {
        unsafe { Timers::new() }
    }
}
