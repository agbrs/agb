//! Button counter example
//!
//! This example demonstrates:
//! - Using embassy tasks for concurrent button handling
//! - Task awaits inputs and sends signals to main
//! - Main handles counter updates and printing
//!
//! Controls:
//! - A/↑/→ buttons: increment counter
//! - B/↓/← buttons: decrement counter
//! - Main loop prints current value every 1 second

#![no_std]
#![no_main]

use embassy_agb::agb::input::Button;
use embassy_agb::input::{AsyncInput, PollingRate};
use embassy_agb::sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_agb::sync::signal::Signal;
use embassy_agb::{input::ButtonEvent, Duration, Spawner, Ticker};

#[derive(Clone, Copy)]
enum CounterAction {
    Increment,
    Decrement,
}

// Signal to communicate counter actions from button task to main
static COUNTER_SIGNAL: Signal<CriticalSectionRawMutex, CounterAction> = Signal::new();

// Task that awaits button inputs and sends signals to main
#[embassy_executor::task]
async fn button_input_task(mut input: AsyncInput) {
    loop {
        let (button, event) = input.wait_for_any_button_press().await;

        // Only handle button presses, not releases
        if event == ButtonEvent::Pressed {
            embassy_agb::agb::println!("Button task: {:?} pressed", button);
            match button {
                Button::A | Button::UP | Button::RIGHT => {
                    COUNTER_SIGNAL.signal(CounterAction::Increment);
                }
                Button::B | Button::DOWN | Button::LEFT => {
                    COUNTER_SIGNAL.signal(CounterAction::Decrement);
                }
                _ => {}
            }
        }
    }
}

#[embassy_agb::main]
async fn main(spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    let _display = gba.display();

    // Enable automatic input polling at 60Hz
    embassy_agb::enable_input_polling(&spawner, PollingRate::Hz60);

    let input = gba.input();

    embassy_agb::agb::println!("Button Counter Example");
    embassy_agb::agb::println!("A/↑/→: increment, B/↓/←: decrement");
    embassy_agb::agb::println!("(Input polling enabled at 60Hz)");

    // Spawn the button input task
    spawner.spawn(button_input_task(input).unwrap());

    // Main loop: handle counter updates and print current value every 1 second
    let mut counter: u8 = 0;
    let mut print_ticker = Ticker::every(Duration::from_secs(1));

    loop {
        // Use select to handle both counter actions and timing
        match embassy_agb::futures::select::select(COUNTER_SIGNAL.wait(), print_ticker.next()).await
        {
            embassy_agb::futures::select::Either::First(action) => match action {
                CounterAction::Increment => {
                    counter = counter.wrapping_add(1);
                }
                CounterAction::Decrement => {
                    counter = counter.wrapping_sub(1);
                }
            },
            embassy_agb::futures::select::Either::Second(_) => {
                embassy_agb::agb::println!("Current counter value: {}", counter);
            }
        }
    }
}
