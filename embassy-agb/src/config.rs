/// Configuration for embassy-agb initialization
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Timer configuration for the time driver
    pub timer: TimerConfig,
}

/// Timer configuration for embassy time driver
#[derive(Debug, Clone)]
pub struct TimerConfig {
    /// Which timer to use for the time driver
    pub timer_number: TimerNumber,
    /// Timer interrupt frequency (overflow amount)
    ///
    /// At 65.536kHz timer frequency:
    /// - 4 counts = ~61μs interrupts, 2 embassy ticks per period (highest precision)
    /// - 16 counts = ~244μs interrupts, 8 embassy ticks per period
    /// - 64 counts = ~977μs interrupts, 32 embassy ticks per period (default)
    /// - 256 counts = ~3.9ms interrupts, 128 embassy ticks per period
    /// - 1024 counts = ~15.6ms interrupts, 512 embassy ticks per period (aligns with 60Hz VBlank)
    pub overflow_amount: u16,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            timer_number: TimerNumber::Timer2,
            overflow_amount: 64, // ~1ms granularity
        }
    }
}

/// Available timers for the time driver
#[derive(Debug, Clone, Copy)]
pub enum TimerNumber {
    /// Timer 0 (used by sound system)
    Timer0,
    /// Timer 1 (used by sound system)
    Timer1,
    /// Timer 2 (default, available for general use)
    Timer2,
    /// Timer 3 (available for general use)
    Timer3,
}
