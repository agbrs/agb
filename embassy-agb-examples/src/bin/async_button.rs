//! Async button example
//!
//! This demonstrates async input handling by waiting for button presses
//! and printing messages. Shows how to use embassy-agb's async input APIs.

#![no_std]
#![no_main]

use embassy_agb::{input::ButtonEvent, Spawner};

#[embassy_agb::main]
async fn main(_spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    let mut input = gba.input();

    embassy_agb::agb::println!("Press any button!");

    loop {
        // Wait for any button press asynchronously
        let (button, event) = input.wait_for_any_button_press().await;

        match event {
            ButtonEvent::Pressed => {
                embassy_agb::agb::println!("Button {:?} pressed!", button);
            }
            ButtonEvent::Released => {
                embassy_agb::agb::println!("Button {:?} released!", button);
            }
        }
    }
}
