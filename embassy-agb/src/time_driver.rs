use core::cell::{Cell, RefCell};
use core::sync::atomic::{Ordering, compiler_fence};
use portable_atomic::AtomicU32;

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use agb::interrupt::{Interrupt, add_interrupt_handler};
use agb::timer::{Divider, Timer};

/// Compile-time timer selection based on feature flags
const TIMER_NUMBER: u16 = if cfg!(feature = "time-driver-timer0") {
    0
} else if cfg!(feature = "time-driver-timer1") {
    1
} else if cfg!(feature = "time-driver-timer2") {
    2
} else if cfg!(feature = "time-driver-timer3") {
    3
} else {
    // This will be caught by the compile-time check below
    0
};

/// Compile-time check to ensure exactly one timer is selected
const _: () = {
    let timer_count =
        0 + if cfg!(feature = "time-driver-timer0") {
            1
        } else {
            0
        } + if cfg!(feature = "time-driver-timer1") {
            1
        } else {
            0
        } + if cfg!(feature = "time-driver-timer2") {
            1
        } else {
            0
        } + if cfg!(feature = "time-driver-timer3") {
            1
        } else {
            0
        };

    if timer_count == 0 {
        panic!(
            "No timer selected for embassy-agb time driver. Enable exactly one of: time-driver-timer0, time-driver-timer1, time-driver-timer2, time-driver-timer3"
        );
    }
    if timer_count > 1 {
        panic!(
            "Multiple timers selected for embassy-agb time driver. Enable exactly one of: time-driver-timer0, time-driver-timer1, time-driver-timer2, time-driver-timer3"
        );
    }
};

/// Get the appropriate timer interrupt based on selected timer
const fn get_timer_interrupt() -> Interrupt {
    match TIMER_NUMBER {
        0 => Interrupt::Timer0,
        1 => Interrupt::Timer1,
        2 => Interrupt::Timer2,
        3 => Interrupt::Timer3,
        _ => unreachable!(),
    }
}

/// Default timer interrupt frequency - provides ~1ms granularity
const DEFAULT_TIMER_OVERFLOW_AMOUNT: u16 = 64;

/// Calculate embassy timestamp from timer periods and current counter value
///
/// The GBA timer runs at 65.536kHz and overflows every timer_overflow_amount counts.
/// Embassy expects 32.768kHz ticks, so we divide hardware ticks by 2.
fn calc_now(
    period: u32,
    counter: u16,
    initial_timer_value: u32,
    timer_overflow_amount: u32,
) -> u64 {
    let overflow_start = 65536 - timer_overflow_amount;

    let hardware_ticks_elapsed = if period == 0 {
        // No overflows yet - calculate ticks from initial timer value
        if counter >= initial_timer_value as u16 {
            (counter - initial_timer_value as u16) as u64
        } else {
            // Counter wrapped around
            ((65536 - initial_timer_value) + counter as u32) as u64
        }
    } else {
        // Calculate ticks from completed periods plus current period progress
        let ticks_from_completed_periods = period as u64 * timer_overflow_amount as u64;

        let ticks_in_current_period = if counter >= overflow_start as u16 {
            (counter - overflow_start as u16) as u64
        } else {
            // Timer wrapped from 65535 to 0
            ((65536 - overflow_start) + counter as u32) as u64
        };

        ticks_from_completed_periods + ticks_in_current_period
    };

    // Convert 65.536kHz hardware ticks to 32.768kHz embassy ticks
    hardware_ticks_elapsed >> 1
}

struct AlarmState {
    timestamp: Cell<u64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

/// GBA Time Driver using configurable timer for embassy-time support
struct GbaTimeDriver {
    period: AtomicU32,
    initial_timer_value: AtomicU32,
    timer_overflow_amount: AtomicU32,
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
    timer: Mutex<CriticalSectionRawMutex, RefCell<Option<Timer>>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: GbaTimeDriver = GbaTimeDriver {
    period: AtomicU32::new(0),
    initial_timer_value: AtomicU32::new(0),
    timer_overflow_amount: AtomicU32::new(DEFAULT_TIMER_OVERFLOW_AMOUNT as u32),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
    timer: Mutex::new(RefCell::new(None)),
});

impl GbaTimeDriver {
    fn init(&'static self) {
        self.init_timer();
    }

    /// Configure timer interrupt frequency
    ///
    /// At 65.536kHz timer frequency:
    /// - 4 counts = ~61μs interrupts, 2 embassy ticks per period (highest precision)
    /// - 16 counts = ~244μs interrupts, 8 embassy ticks per period
    /// - 64 counts = ~977μs interrupts, 32 embassy ticks per period (default)
    /// - 256 counts = ~3.9ms interrupts, 128 embassy ticks per period
    pub fn set_timer_frequency(&self, overflow_amount: u16) {
        self.timer_overflow_amount
            .store(overflow_amount as u32, Ordering::Relaxed);
    }

    fn init_timer(&self) {
        critical_section::with(|cs| {
            let mut timer_ref = self.timer.borrow(cs).borrow_mut();

            // Configure selected timer for embassy timing
            let all_timers = unsafe { agb::timer::AllTimers::new() };
            let mut timer = match TIMER_NUMBER {
                0 => all_timers.timer0,
                1 => all_timers.timer1,
                2 => all_timers.timer2,
                3 => all_timers.timer3,
                _ => unreachable!(),
            };

            let overflow_amount = self.timer_overflow_amount.load(Ordering::Relaxed) as u16;
            timer
                .set_divider(Divider::Divider256) // 65.536kHz
                .set_overflow_amount(overflow_amount)
                .set_interrupt(true)
                .set_enabled(true);

            // Capture initial timer value for precise timing calculations
            let initial_value = timer.value();
            self.initial_timer_value
                .store(initial_value as u32, Ordering::Relaxed);

            // Install interrupt handler for selected timer
            let handler = unsafe {
                add_interrupt_handler(get_timer_interrupt(), |_| {
                    DRIVER.on_interrupt();
                })
            };
            core::mem::forget(handler);

            *timer_ref = Some(timer);
        });
    }

    fn on_interrupt(&self) {
        self.period.fetch_add(1, Ordering::Relaxed);
        critical_section::with(|cs| {
            self.trigger_alarm(cs);
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(u64::MAX);

        let mut next = self
            .queue
            .borrow(cs)
            .borrow_mut()
            .next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self
                .queue
                .borrow(cs)
                .borrow_mut()
                .next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        let now = self.now();
        if timestamp <= now {
            alarm.timestamp.set(u64::MAX);
            false
        } else {
            true
        }
    }

    fn read_timer_value(&self) -> u16 {
        critical_section::with(|cs| {
            let timer_ref = self.timer.borrow(cs).borrow();
            if let Some(timer) = timer_ref.as_ref() {
                timer.value()
            } else {
                0
            }
        })
    }
}

impl Driver for GbaTimeDriver {
    fn now(&self) -> u64 {
        let period = self.period.load(Ordering::Relaxed);
        let initial_timer_value = self.initial_timer_value.load(Ordering::Relaxed);
        let timer_overflow_amount = self.timer_overflow_amount.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = self.read_timer_value();
        calc_now(period, counter, initial_timer_value, timer_overflow_amount)
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        })
    }
}

/// Initialize the time driver
pub(crate) fn init() {
    DRIVER.init();
}

/// Configure the timer interrupt frequency
///
/// This must be called before using any embassy-time functionality.
/// The configuration is typically set through the Config struct in init().
pub(crate) fn configure_timer_frequency(overflow_amount: u16) {
    DRIVER.set_timer_frequency(overflow_amount);
}
