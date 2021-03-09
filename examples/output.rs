#![no_std]
#![feature(start)]

use gba::display::vblank;

extern crate gba;
#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = gba::Gba::new();
    let mut mgba = gba::mgba::Mgba::new().unwrap();

    let vblank = gba.display.vblank.get();

    let mut count = 0;
    loop {
        vblank.wait_for_VBlank();

        mgba.print(
            format_args!("Hello, world, frame = {}", count),
            gba::mgba::DebugLevel::Info,
        )
        .unwrap();

        count += 1;
    }
}
