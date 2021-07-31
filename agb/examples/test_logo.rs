#![no_std]
#![no_main]

extern crate agb;

use agb::display::example_logo;
use agb::display::tiled0::Map;

#[no_mangle]
pub fn main() -> ! {
    let mut gba = agb::Gba::new();
    let mut gfx = gba.display.video.tiled0();

    example_logo::display_logo(&mut gfx);

    loop {}
}
