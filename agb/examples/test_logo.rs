#![no_std]
#![no_main]

use agb::display::{
    example_logo,
    tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (mut gfx, mut vram) = gba.display.video.tiled();

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    example_logo::display_logo(&mut map, &mut vram);
    map.commit();

    let mut bg_iter = gfx.iter();
    map.show(&mut bg_iter);
    bg_iter.commit(&mut vram);

    loop {}
}
