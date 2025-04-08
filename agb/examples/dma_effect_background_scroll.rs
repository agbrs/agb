#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use agb::{
    display::{
        HEIGHT, example_logo,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
    },
    dma::HBlankDmaDefinition,
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    example_logo::display_logo(&mut map);

    let offsets: Box<[_]> = (0..(32 * 16 + HEIGHT as u16)).collect();

    let mut frame_count = 0;

    loop {
        let mut frame = gfx.frame();
        let background_id = map.show(&mut frame);

        frame_count += 1;
        if frame_count > offsets.len() - HEIGHT as usize {
            frame_count = 0;
        }

        HBlankDmaDefinition::new(background_id.x_scroll_dma(), &offsets[frame_count..])
            .show(&mut frame);

        frame.commit();
    }
}
