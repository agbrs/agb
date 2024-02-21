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

    let offsets: Box<[_]> = (0..160 * 2).collect();

    let mut frame = 0;

    loop {
        let _x_scroll_transfer =
            unsafe { dma.hblank_transfer(&map.x_scroll_dma(), &offsets[frame..]) };

        vblank.wait_for_vblank();
        frame += 1;
        if frame > 160 {
            frame = 0;
        }
    }
}
