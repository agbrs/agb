#![no_std]
#![no_main]

extern crate gba;
#[no_mangle]
pub extern "C" fn main() -> ! {
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
