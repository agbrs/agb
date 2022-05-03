#![no_std]
#![no_main]

use agb::display::{example_logo, tiled::RegularBackgroundSize};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();

    let mut map = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
    );

    example_logo::display_logo(&mut map, &mut vram);

    loop {}
}
