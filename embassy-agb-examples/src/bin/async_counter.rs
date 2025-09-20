//! Async counter example
//!
//! This demonstrates basic async timing by displaying a counter that increments
//! every second using embassy-time. Shows the simplest possible async agb application.

#![no_std]
#![no_main]

use embassy_agb::{Duration, Instant, Spawner, Ticker};

#[embassy_agb::main]
async fn main(_spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    let mut display = gba.display();

    let start_time = Instant::now();
    let mut counter = 0u32;
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        // Wait for VBlank and get a frame
        let _frame = display.frame().await;

        // Calculate time since boot
        let elapsed = Instant::now() - start_time;
        let elapsed_secs = elapsed.as_secs();
        let elapsed_millis = elapsed.as_millis() % 1000;

        // Display counter and time since boot
        embassy_agb::agb::println!(
            "Counter: {} | Boot time: {}.{:03}s",
            counter,
            elapsed_secs,
            elapsed_millis
        );

        counter += 1;

        // Wait for next tick - this maintains precise 1-second intervals
        ticker.next().await;
    }
}
