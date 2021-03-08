#![no_std]
#![feature(start)]

extern crate gba;
use gba::{display, syscall};

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = gba::Gba::new();
    let mut bitmap = gba.display.video.bitmap3();

    for x in 0..display::WIDTH {
        let y = syscall::sqrt(x << 6);
        let y = (display::HEIGHT - y).clamp(0, display::HEIGHT - 1);
        bitmap.draw_point(x, y, 0x001F);
    }

    loop {
        syscall::halt();
    }
}
