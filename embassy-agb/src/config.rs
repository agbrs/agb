/// Configuration for embassy-agb initialization
#[derive(Debug, Clone)]
pub struct Config {
    /// Timer configuration for the time driver
    pub timer: TimerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timer: TimerConfig::default(),
        }
    }
}

/// Timer configuration for embassy time driver
#[derive(Debug, Clone)]
pub struct TimerConfig {
    /// Which timer to use for the time driver
    pub timer_number: TimerNumber,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            timer_number: TimerNumber::Timer0,
        }
    }
}

/// Available timers for the time driver
#[derive(Debug, Clone, Copy)]
pub enum TimerNumber {
    /// Timer 0 (recommended for time driver)
    Timer0,
    /// Timer 1
    Timer1,
}
