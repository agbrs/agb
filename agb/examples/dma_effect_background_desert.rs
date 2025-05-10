//! This uses the vertical offset DMA register for a regular background to
//! create a sort of heat wave effect good for desert backgrounds.
#![no_std]
#![no_main]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use agb::{
    display::{
        HEIGHT,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    dma::HBlankDma,
    fixnum::Num,
    include_background_gfx,
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let map = get_logo();

    // Create sine wave through as the offset to create the stretchy wave.
    // We calculate this in advance as a performance improvement.
    let offsets: Box<[Num<i32, 8>]> = (0..(32 * 8 + HEIGHT))
        .map(|y| (Num::new(y) / 16).sin())
        .collect();

    let mut frame_count = 0;

    loop {
        let mut frame = gfx.frame();
        let background_id = map.show(&mut frame);

        // Loop the animation if we need to
        frame_count += 1;
        if frame_count > offsets.len() - HEIGHT as usize {
            frame_count = 0;
        }

        let offsets: Vec<_> = (0..160i16)
            .map(|y| (offsets[frame_count + y as usize] * 3).floor() as i16)
            .map(|offset| offset as u16)
            .collect();

        HBlankDma::new(background_id.y_scroll_dma(), &offsets).show(&mut frame);

        frame.commit();
    }
}

fn get_logo() -> RegularBackgroundTiles {
    include_background_gfx!(mod backgrounds, LOGO => "examples/gfx/test_logo.aseprite");

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);
    map.fill_with(&backgrounds::LOGO);

    map
}
