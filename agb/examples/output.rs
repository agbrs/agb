#![no_std]
#![feature(start)]

extern crate agb;
#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = agb::Gba::new();
    let mut mgba = agb::mgba::Mgba::new().unwrap();

    let vblank = gba.display.vblank.get();

    let mut count = 0;
    loop {
        vblank.wait_for_VBlank();

        mgba.print(
            format_args!("Hello, world, frame = {}", count),
            agb::mgba::DebugLevel::Info,
        )
        .unwrap();

        count += 1;
    }
}
