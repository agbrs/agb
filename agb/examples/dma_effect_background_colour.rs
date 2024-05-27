#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use agb::{
    display::{
        example_logo,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
    },
    interrupt::VBlank,
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (mut gfx, mut vram) = gba.display.video.tiled();

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    example_logo::display_logo_basic(&mut map, &mut vram);
    map.commit();

    let vblank = VBlank::get();

    let colours: Box<[_]> = (0..160).map(|i| ((i * 0xffff) / 160) as u16).collect();

    let dma = gba.dma.dma().dma0;

    let background_colour = 0x732b; // generated using `https://agbrs.dev/colour`
    let background_colour_index = vram
        .find_colour_index_16(0, background_colour)
        .expect("Should contain colour 0x732b");

    loop {
        let _background_color_transfer = unsafe {
            dma.hblank_transfer(
                &vram.background_palette_colour_dma(0, background_colour_index),
                &colours,
            )
        };

        vblank.wait_for_vblank();
        let mut bg_iter = gfx.iter();
        map.show(&mut bg_iter);
        bg_iter.commit(&mut vram);
    }
}
