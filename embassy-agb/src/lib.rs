#![no_std]
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
//! - Async input handling (button press events)
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
//! use embassy_agb::{Duration, Ticker};
//! use embassy_executor::Spawner;
//!
//! #[embassy_agb::main]
//! async fn main(_spawner: Spawner) -> ! {
//!     let mut gba = embassy_agb::init(Default::default());
//!     let mut display = gba.display();
//!     
//!     let mut counter = 0u32;
//!     let mut ticker = Ticker::every(Duration::from_secs(1));
//!     
//!     loop {
//!         display.wait_for_vblank().await;
//!         agb::println!("Counter: {}", counter);
//!         counter += 1;
//!         ticker.next().await; // Precise 1-second intervals
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

pub mod config;
pub use config::*;

#[cfg(feature = "_time-driver")]
mod time_driver;

#[cfg(feature = "executor")]
mod executor;
#[cfg(feature = "executor")]
pub use executor::*;

pub mod display;
pub mod input;
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
    peripherals: Peripherals,
    _config: Config,
}

impl InitializedGba {
    /// Get the display peripheral for async operations
    pub fn display(&mut self) -> display::AsyncDisplay {
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

    /// Get access to the underlying agb::Gba for compatibility
    pub fn agb(&mut self) -> &mut agb::Gba {
        self.gba
    }
}
