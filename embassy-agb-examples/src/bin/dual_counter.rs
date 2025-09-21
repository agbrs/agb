//! Dual counter example
//!
//! This example demonstrates running two counters at different rates:
//! - Main counter: increments every 500ms
//! - Task counter: increments every 200ms
//!
//! This shows how to use embassy-agb to run concurrent async tasks
//! with different timing requirements.

#![no_std]
#![no_main]

use embassy_agb::{Duration, Spawner, Ticker};

// Async task that counts at a faster rate
#[embassy_executor::task]
async fn fast_counter_task() {
    let mut counter = 0u32;
    let mut ticker = Ticker::every(Duration::from_millis(200));

    loop {
        ticker.next().await;

        embassy_agb::agb::println!("  Task counter: {}", counter);
        counter += 1;
    }
}

#[embassy_agb::main]
async fn main(spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    let _display = gba.display();

    embassy_agb::agb::println!("Starting dual counters:");
    embassy_agb::agb::println!("Main: 500ms intervals");
    embassy_agb::agb::println!("Task: 200ms intervals");
    embassy_agb::agb::println!("");

    // Spawn the fast counter task
    spawner.spawn(fast_counter_task().unwrap());

    // Main counter runs at a slower rate
    let mut main_counter = 0u32;
    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        ticker.next().await;

        embassy_agb::agb::println!("Main counter: {}", main_counter);
        main_counter += 1;
    }
}
