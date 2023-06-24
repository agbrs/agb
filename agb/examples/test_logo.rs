#![no_std]
#![no_main]

use agb::display::{
    example_logo,
    tiled::{RegularBackgroundSize, TileFormat},
    video::Tiled0Vram,
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, vram) = &mut *gba.display.video.get::<Tiled0Vram>();

    let mut map = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    example_logo::display_logo(&mut map, vram);

    loop {}
}
