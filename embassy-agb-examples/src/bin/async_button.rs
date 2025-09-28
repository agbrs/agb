#![no_std]
#![no_main]

use embassy_agb::{input::{ButtonEvent, InputConfig}, Spawner};

#[embassy_agb::main]
async fn main(spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    let mut input = gba.input();

    // Configure input polling at 60Hz (matches VBlank rate)
    let input_config = InputConfig { poll_rate: 60 };

    // Spawn the input polling task
    spawner.spawn(embassy_agb::input::input_polling_task(input_config).unwrap());

    embassy_agb::agb::println!("Press any button!");

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
