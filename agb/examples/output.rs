#![no_std]
#![no_main]

extern crate agb;
#[no_mangle]
pub fn main() -> ! {
    let mut gba = agb::Gba::new();

    let vblank = agb::interrupt::VBlank::new();

    let mut count = 0;
    loop {
        vblank.wait_for_vblank();

        agb::println!("Hello, world, frame = {}", count);

        count += 1;
    }
}
