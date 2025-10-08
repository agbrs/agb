#![no_std]
// This appears to be needed for testing to work
#![cfg_attr(any(test, feature = "testing"), no_main)]
#![cfg_attr(any(test, feature = "testing"), feature(custom_test_frameworks))]
#![cfg_attr(
    any(test, feature = "testing"),
    test_runner(agb::test_runner::test_runner)
)]
#![cfg_attr(
    any(test, feature = "testing"),
    reexport_test_harness_main = "test_main"
)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! # Embassy async support for agb
//!
//! This crate provides async/await support for Game Boy Advance development using the embassy executor.
//! It integrates with the existing agb library to provide async APIs for display, input, sound, and timing.
//!
//! ## Features
//!
//! - Async display operations (VBlank waiting, DMA transfers)
//! - Async input handling (button press events) with automatic polling
//! - Async sound mixing
//! - Embassy time integration with GBA timers
//! - Task spawning and management
//!
//! ## Example
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! use embassy_agb::{input::{ButtonEvent, PollingRate}};
//! use embassy_executor::Spawner;
//!
//! #[embassy_agb::main]
//! async fn main(spawner: Spawner) -> ! {
//!     let mut gba = embassy_agb::init(Default::default());
//!     
//!     // Enable automatic input polling at 60Hz
//!     embassy_agb::enable_input_polling(&spawner, PollingRate::Hz60);
//!     
//!     let mut input = gba.input();
//!     let mut display = gba.display();
//!     
//!     loop {
//!         display.wait_for_vblank().await;
//!         
//!         // Async input handling - no manual polling needed!
//!         let (button, event) = input.wait_for_any_button_press().await;
//!         if event == ButtonEvent::Pressed {
//!             agb::println!("Button {:?} pressed!", button);
//!         }
//!     }
//! }
//! ```

// Include generated code
include!(concat!(env!("OUT_DIR"), "/_generated.rs"));

#[cfg(feature = "executor")]
pub use embassy_executor::Spawner;

// Re-export our macros
pub use embassy_agb_macros::{main, task};

#[cfg(feature = "time")]
pub use embassy_time as time;

#[cfg(feature = "time")]
pub use embassy_time::{Duration, Instant, Ticker, Timer};

pub use embassy_futures as futures;
pub use embassy_sync as sync;

// Re-export agb for convenience
pub use agb;

/// Configuration types for embassy-agb
pub mod config;
pub use config::*;

#[cfg(feature = "_time-driver")]
mod time_driver;

#[cfg(feature = "executor")]
mod executor;
#[cfg(feature = "executor")]
pub use executor::*;

/// Async display utilities
pub mod display;
pub mod input;
/// Async sound utilities
pub mod sound;

/// Internal utilities (do not use directly)
#[doc(hidden)]
pub mod _internal;

/// Initialize the embassy-agb HAL with the given configuration.
///
/// This function must be called once before using any embassy-agb functionality.
/// It initializes the underlying agb library and sets up embassy integration.
///
/// # Example
///
/// ```rust,no_run
/// let gba = embassy_agb::init(Default::default());
/// ```
pub fn init(config: Config) -> InitializedGba {
    // Get the agb instance from internal storage (set by macro)
    let gba = unsafe { _internal::get_agb_instance() };

    // Configure the time driver with user settings
    #[cfg(feature = "_time-driver")]
    time_driver::configure_timer_frequency(config.timer.overflow_amount);

    // Take peripherals
    let peripherals = Peripherals::take();

    InitializedGba {
        gba,
        peripherals,
        _config: config,
    }
}

/// The initialized GBA with embassy integration
pub struct InitializedGba {
    gba: &'static mut agb::Gba,
    #[allow(dead_code)]
    peripherals: Peripherals,
    _config: Config,
}

impl InitializedGba {
    /// Get the display peripheral for async operations
    pub fn display(&mut self) -> display::AsyncDisplay<'_> {
        display::AsyncDisplay::new(&mut self.gba.graphics)
    }

    /// Get the mixer peripheral for async operations
    pub fn mixer(&mut self, frequency: agb::sound::mixer::Frequency) -> sound::AsyncMixer<'_> {
        sound::AsyncMixer::new(&mut self.gba.mixer, frequency)
    }

    /// Get the input peripheral for async operations
    pub fn input(&mut self) -> input::AsyncInput {
        input::AsyncInput::new()
    }

    /// Get the input peripheral for async operations with custom configuration
    pub fn input_with_config(&mut self, config: input::InputConfig) -> input::AsyncInput {
        input::AsyncInput::with_config(config)
    }

    /// Get access to the underlying agb::Gba for compatibility
    pub fn agb(&mut self) -> &mut agb::Gba {
        self.gba
    }
}

/// Enable automatic input polling with the given polling rate.
///
/// This function should be called once at startup to automatically spawn
/// the input polling task. If not called, input methods will still work
/// but will use polling-based approach instead of interrupt-driven.
///
/// # Example
///
/// ```rust,no_run
/// use embassy_agb::input::PollingRate;
///
/// #[embassy_agb::main]
/// async fn main(spawner: Spawner) -> ! {
///     let mut gba = embassy_agb::init(Default::default());
///     
///     // Enable automatic input polling at 60Hz
///     embassy_agb::enable_input_polling(&spawner, PollingRate::Hz60);
///     
///     let mut input = gba.input();
///     // ... rest of your code
/// }
/// ```
#[cfg(all(feature = "time", feature = "executor"))]
pub fn enable_input_polling(spawner: &Spawner, rate: input::PollingRate) {
    let config = input::InputConfig::from(rate);
    if let Ok(token) = input::input_polling_task(config) {
        spawner.spawn(token);
    }
}
