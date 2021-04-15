#![no_std]
#![feature(start)]

extern crate gba;

use gba::sound;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let gba = gba::Gba::new();

    gba.sound.enable();

    let sweep_settings = sound::SweepSettings::default();
    gba.sound.channel1().play_sound(
        1024,
        Some(0),
        &sweep_settings,
        &sound::EnvelopeSettings::default(),
        sound::DutyCycle::Half,
    );

    loop {}
}
