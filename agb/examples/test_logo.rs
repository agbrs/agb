#![no_std]
#![no_main]

use agb::display::example_logo;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.display.video.tiled0();

    let mut map = gfx.background();
    let mut vram = gfx.vram;

    example_logo::display_logo(&mut map, &mut vram);

    loop {}
}
