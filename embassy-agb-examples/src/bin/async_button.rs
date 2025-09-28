#![no_std]
#![no_main]

use embassy_agb::{input::ButtonEvent, Spawner};

#[embassy_agb::main]
async fn main(spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    
    // Enable automatic input polling at 60Hz
    embassy_agb::enable_input_polling_60hz(&spawner);
    
    let mut input = gba.input();

    embassy_agb::agb::println!("Press any button! (Input polling enabled at 60Hz)");

    loop {
        let (button, event) = input.wait_for_any_button_press().await;

        match event {
            ButtonEvent::Pressed => {
                embassy_agb::agb::println!("{:?} pressed", button);
            }
            ButtonEvent::Released => {
                embassy_agb::agb::println!("{:?} released", button);
            }
        }
    }
}
