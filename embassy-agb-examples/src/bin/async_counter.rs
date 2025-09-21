//! Async counter example
//!
//! This demonstrates precise async timing by displaying a counter that increments
//! every 10ms using embassy-time. Shows ~1ms timing granularity with the default
//! 64-count timer configuration (~977μs interrupts).

#![no_std]
#![no_main]

use embassy_agb::{config::Config, Duration, Instant, Spawner, Ticker};

#[embassy_agb::main]
async fn main(_spawner: Spawner) -> ! {
    // Configure timer for ~1ms granularity (default is 64 counts)
    // For higher precision, use smaller values like 16 (~244μs) or 4 (~61μs)
    let config = Config::default(); // Uses 64 counts = ~977μs interrupts
    let mut gba = embassy_agb::init(config);
    let _display = gba.display();

    let start_time = Instant::now();
    let mut counter = 0u32;
    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        // Wait for next tick - this maintains precise 100ms intervals
        ticker.next().await;

        // Calculate time since boot
        let elapsed = Instant::now() - start_time;
        let elapsed_millis = elapsed.as_millis();
        let elapsed_ticks = elapsed.as_ticks();

        // Display counter and time since boot - showing millisecond precision
        embassy_agb::agb::println!(
            "Counter: {} | Elapsed: {}ms | Ticks: {}",
            counter,
            elapsed_millis,
            elapsed_ticks
        );

        counter += 1;
    }
}
