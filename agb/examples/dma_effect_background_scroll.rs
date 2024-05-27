#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use agb::{
    display::{
        HEIGHT, example_logo,
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

    example_logo::display_logo(&mut map, &mut vram);
    map.commit();

    let vblank = VBlank::get();

    let dma = gba.dma.dma().dma0;
    let offsets: Box<[_]> = (0..(32 * 16 + HEIGHT as u16)).collect();

    let mut frame = 0;

    loop {
        let mut bg_iter = gfx.iter();
        let background_id = map.show(&mut bg_iter);
        bg_iter.commit(&mut vram);

        let _x_scroll_transfer =
            dma.hblank_transfer(&background_id.x_scroll_dma(), &offsets[frame..]);

        vblank.wait_for_vblank();

        frame += 1;
        if frame > offsets.len() - HEIGHT as usize {
            frame = 0;
        }
    }
}
