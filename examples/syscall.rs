#![no_std]
#![no_main]

extern crate gba;
use gba::{display, syscall};

#[no_mangle]
pub extern "C" fn main() -> ! {
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
