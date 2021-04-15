#![no_std]
#![feature(start)]

extern crate gba;

use gba::sound;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = gba::Gba::new();

    gba.sound.enable();

    let sweep_settings = sound::SweepSettings::new(3, sound::SoundDirection::Decrease, 7);
    gba.sound.channel1().play_sound(&sweep_settings);

    loop {}
}
