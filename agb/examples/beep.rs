#![no_std]
#![no_main]

extern crate agb;

use agb::sound;

#[agb::entry]
fn main() -> ! {
    let gba = agb::Gba::new();

    gba.sound.enable();

    let sweep_settings = sound::dmg::SweepSettings::default();
    gba.sound.channel1().play_sound(
        1024,
        Some(0),
        &sweep_settings,
        &sound::dmg::EnvelopeSettings::default(),
        sound::dmg::DutyCycle::Half,
    );

    gba.sound.channel2().play_sound(
        1524,
        Some(0),
        &sound::dmg::EnvelopeSettings::default(),
        sound::dmg::DutyCycle::Half,
    );

    gba.sound.noise().play_sound(
        Some(0),
        &sound::dmg::EnvelopeSettings::default(),
        4,
        false,
        1,
    );

    loop {}
}
