//! Button counter example
//!
//! This example demonstrates:
//! - Using embassy tasks for concurrent button handling
//! - Sharing state between tasks using atomic operations
//! - Main loop printing values at regular intervals
//!
//! Controls:
//! - A button: increment counter
//! - B button: decrement counter
//! - Main loop prints current value every 1 second

#![no_std]
#![no_main]

use embassy_agb::{input::ButtonEvent, Duration, Spawner, Ticker};
use embassy_agb::agb::input::Button;
use portable_atomic::{AtomicU8, Ordering};
use embassy_agb::sync::signal::Signal;
use embassy_agb::sync::blocking_mutex::raw::CriticalSectionRawMutex;

// Shared counter between tasks
static COUNTER: AtomicU8 = AtomicU8::new(0);

// Signal to communicate button events from main to task
static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, (Button, ButtonEvent)> = Signal::new();

// Task that processes button events received via signal
#[embassy_executor::task]
async fn button_processor_task() {
    loop {
        // Wait for button event from main
        let (button, event) = BUTTON_SIGNAL.wait().await;
        
        // Only handle button presses, not releases
        if event == ButtonEvent::Pressed {
            match button {
                Button::A | Button::UP | Button::RIGHT => {
                    let old_value = COUNTER.fetch_add(1, Ordering::SeqCst);
                    embassy_agb::agb::println!("Task processed A: {} -> {}", old_value, old_value.wrapping_add(1));
                }
                Button::B | Button::DOWN | Button::LEFT => {
                    let old_value = COUNTER.fetch_sub(1, Ordering::SeqCst);
                    embassy_agb::agb::println!("Task processed B: {} -> {}", old_value, old_value.wrapping_sub(1));
                }
                _ => {
                    embassy_agb::agb::println!("Task received other button: {:?}", button);
                }
            }
        }
    }
}

#[embassy_agb::main]
async fn main(spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    let _display = gba.display();

    let mut input = gba.input();

    embassy_agb::agb::println!("Button Counter Example");
    embassy_agb::agb::println!("A: increment, B: decrement");
    embassy_agb::agb::println!("Current value printed every 1s");
    embassy_agb::agb::println!("");

    // Spawn the button processor task
    spawner.spawn(button_processor_task().unwrap());

    // Main loop: handle input and print current value every 1 second
    let mut print_ticker = Ticker::every(Duration::from_secs(1));

    loop {
        // Use select to handle both input and timing
        embassy_agb::futures::select::select(
            async {
                let (button, event) = input.wait_for_any_button_press().await;
                // Send button event to processing task
                BUTTON_SIGNAL.signal((button, event));
            },
            async {
                print_ticker.next().await;
                let current_value = COUNTER.load(Ordering::SeqCst);
                embassy_agb::agb::println!("Current counter value: {}", current_value);
            }
        ).await;
    }
}