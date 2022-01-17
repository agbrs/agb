#![no_std]
#![no_main]

use agb::display::example_logo;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.display.video.tiled0();

    example_logo::display_logo(&mut gfx);

    loop {}
}
