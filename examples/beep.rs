#![no_std]
#![feature(start)]

extern crate gba;

use gba::display;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = gba::Gba::new();
    let mut bitmap = gba.display.video.bitmap3();
    let vblank = gba.display.vblank.get();

    gba.sound.enable();
    gba.sound.channel1().play_sound();

    loop {}
}
