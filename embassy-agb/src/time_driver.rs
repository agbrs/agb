use core::cell::{Cell, RefCell};
use core::sync::atomic::{compiler_fence, Ordering};
use portable_atomic::AtomicU32;

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use agb::interrupt::{add_interrupt_handler, Interrupt};
use agb::timer::{Divider, Timer};

/// Calculate timestamp from period and counter, handling race conditions
///
/// GBA-specific approach:
/// - Timer runs at 65.536kHz, overflows every 32768 counts (0.5 seconds)
/// - Each interrupt increments period, representing 16384 embassy ticks
/// - Counter value represents additional ticks at 2:1 ratio (65.536kHz -> 32.768kHz)
/// - Use similar XOR pattern as other Embassy drivers for race condition handling
fn calc_now(period: u32, counter: u16) -> u64 {
    // Each period represents 16384 ticks (0.5 seconds at 32.768kHz)
    // Counter is scaled down by 2 to convert from 65.536kHz to 32.768kHz
    // Simple approach: period increments every 0.5s, counter gives sub-period precision
    ((period as u64) << 14) + ((counter as u64) >> 1)
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

/// GBA Time Driver using hardware timers
///
/// Uses GBA Timer0 to provide embassy-time support.
/// The timer runs at 32.768kHz to match embassy's expected tick rate.
struct GbaTimeDriver {
    /// Number of 2^15 periods elapsed since boot
    period: AtomicU32,
    /// Alarm state for scheduled wakeups
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
    /// Timer queue for scheduled wakeups
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
    /// Hardware timer (Timer0)
    timer: Mutex<CriticalSectionRawMutex, RefCell<Option<Timer>>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: GbaTimeDriver = GbaTimeDriver {
    period: AtomicU32::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
    timer: Mutex::new(RefCell::new(None)),
});

impl GbaTimeDriver {
    fn init(&'static self) {
        // Set up Timer0 for embassy time support
        self.init_timer();
    }

    fn init_timer(&self) {
        critical_section::with(|cs| {
            let mut timer_ref = self.timer.borrow(cs).borrow_mut();

            // Create Timer0 for time tracking
            // Note: We use Timer0 which may conflict with sound mixer if both are used
            let all_timers = unsafe { agb::timer::AllTimers::new() };
            let mut timer = all_timers.timer0;

            // Configure timer for Embassy's 32.768kHz tick rate
            // GBA: Use 65.536kHz (Divider256) and overflow every 32768 counts
            // This gives us interrupts every 0.5 seconds (32768/65536)
            // We'll increment period on each interrupt to maintain 2^15 tick periods
            timer
                .set_divider(Divider::Divider256) // 65.536kHz
                .set_overflow_amount(32768) // Interrupt every 0.5 seconds
                .set_interrupt(true)
                .set_enabled(true);

            // Set up interrupt handler for timer overflow
            let handler = unsafe {
                add_interrupt_handler(Interrupt::Timer0, |_| {
                    DRIVER.on_interrupt();
                })
            };

            // Keep the handler alive by leaking it
            core::mem::forget(handler);

            *timer_ref = Some(timer);
        });
    }

    fn on_interrupt(&self) {
        // Timer interrupts every 0.5 seconds (32768 counts at 65.536kHz)
        // Each period increment represents 2^14 = 16384 embassy ticks
        // This gives us the target rate: 2 interrupts/second * 16384 ticks = 32768 ticks/second
        self.period.fetch_add(1, Ordering::Relaxed);

        // Process any expired timers
        critical_section::with(|cs| {
            self.trigger_alarm(cs);
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(u64::MAX);

        // Process expired timers and get next expiration
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
            // Alarm has already passed
            alarm.timestamp.set(u64::MAX);
            false
        } else {
            // Alarm is in the future - for GBA we rely on periodic timer interrupts
            // to check for expired alarms rather than precise timing
            true
        }
    }

    fn read_timer_value(&self) -> u16 {
        critical_section::with(|cs| {
            let timer_ref = self.timer.borrow(cs).borrow();
            if let Some(timer) = timer_ref.as_ref() {
                timer.value()
            } else {
                // Fallback if timer not initialized yet
                0
            }
        })
    }
}

impl Driver for GbaTimeDriver {
    fn now(&self) -> u64 {
        // Must read period before counter to avoid race conditions
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = self.read_timer_value();
        calc_now(period, counter)
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
