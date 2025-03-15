#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use agb::display::{
    HEIGHT, example_logo,
    tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.display.graphics.get();

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    example_logo::display_logo(&mut map);

    let mut dma = gba.dma.dma().dma0;
    let offsets: Box<[_]> = (0..(32 * 16 + HEIGHT as u16)).collect();

    let mut frame_count = 0;

    let mut x_scroll_transfer = None;

    loop {
        let mut frame = gfx.frame();
        let background_id = map.show(&mut frame);

        frame_count += 1;
        if frame_count > offsets.len() - HEIGHT as usize {
            frame_count = 0;
        }

        frame.commit();

        drop(x_scroll_transfer);
        x_scroll_transfer = Some(unsafe {
            dma.hblank_transfer(&background_id.x_scroll_dma(), &offsets[frame_count..])
        });
    }
}
