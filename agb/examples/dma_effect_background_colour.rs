#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use agb::{
    display::{
        example_logo,
        tiled::{RegularBackgroundSize, TileFormat},
    },
    interrupt::VBlank,
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();

    let mut map = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let dma = gba.dma.dma().dma0;

    example_logo::display_logo(&mut map, &mut vram);

    let vblank = VBlank::get();

    let colours: Box<[_]> = (0..160).map(|i| ((i * 0xffff) / 160) as u16).collect();

    let mut frame = 0;

    loop {
        // hardcoding palette index 2 here which you wouldn't want to do in a real example (instead, look for
        // the colour you want to replace)
        let _background_color_transfer =
            unsafe { dma.hblank_transfer(&vram.background_palette_colour_dma(0, 2), &colours) };

        vblank.wait_for_vblank();
        frame += 1;
        if frame > 160 {
            frame = 0;
        }
    }
}
