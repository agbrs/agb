//! Dual counter example
//!
//! This example demonstrates running two counters at different rates:
//! - Main counter: increments every 500ms
//! - Task counter: increments every 200ms
//!
//! This shows how to use embassy-agb to run concurrent async tasks
//! with different timing requirements, while also displaying precise
//! timing information.

#![no_std]
#![no_main]

use embassy_agb::{config::Config, Duration, Instant, Spawner, Ticker};

// Async task that counts at a faster rate
#[embassy_executor::task]
async fn fast_counter_task(start_time: Instant) {
    let mut counter = 0u32;
    let mut ticker = Ticker::every(Duration::from_millis(200));

    loop {
        ticker.next().await;

        // Calculate time since boot
        let elapsed = Instant::now() - start_time;
        let elapsed_millis = elapsed.as_millis();
        let elapsed_ticks = elapsed.as_ticks();

        embassy_agb::agb::println!(
            "Task counter: {:3} | Elapsed: {:5}ms | Ticks: {}",
            counter,
            elapsed_millis,
            elapsed_ticks
        );
        counter += 1;
    }
}

#[embassy_agb::main]
async fn main(spawner: Spawner) -> ! {
    // Configure timer for ~1ms granularity (default is 64 counts)
    // For higher precision, use smaller values like 16 (~244μs) or 4 (~61μs)
    let config = Config::default(); // Uses 64 counts = ~977μs interrupts
    let mut gba = embassy_agb::init(config);
    let _display = gba.display();

    let start_time = Instant::now();

    embassy_agb::agb::println!("Starting dual counters:");
    embassy_agb::agb::println!("Main: 500ms intervals");
    embassy_agb::agb::println!("Task: 200ms intervals");
    embassy_agb::agb::println!("");

    // Spawn the fast counter task
    spawner.spawn(fast_counter_task(start_time).unwrap());

    // Main counter runs at a slower rate
    let mut main_counter = 0u32;
    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        // Wait for next tick - this maintains precise 500ms intervals
        ticker.next().await;

        // Calculate time since boot
        let elapsed = Instant::now() - start_time;
        let elapsed_millis = elapsed.as_millis();
        let elapsed_ticks = elapsed.as_ticks();

        // Display main counter and time since boot - showing millisecond precision
        embassy_agb::agb::println!(
            "Main counter: {:3} | Elapsed: {:5}ms | Ticks: {}",
            main_counter,
            elapsed_millis,
            elapsed_ticks
        );

        main_counter += 1;
    }
}
