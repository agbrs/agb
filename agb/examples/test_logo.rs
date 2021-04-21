#![no_std]
#![feature(start)]

extern crate agb;

use agb::display::{example_logo, tiled0};

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = agb::Gba::new();
    let mut gfx = gba.display.video.tiled0();

    gfx.set_background_palettes(example_logo::PALETTE_DATA);
    gfx.set_background_tilemap(example_logo::TILE_DATA);

    gfx.background_0.enable();
    gfx.background_0
        .set_background_size(tiled0::BackgroundSize::S32x32);
    gfx.background_0
        .set_colour_mode(tiled0::ColourMode::FourBitPerPixel);

    gfx.background_0.set_screen_base_block(20);

    let mut entries: [u16; 32 * 20] = [0; 32 * 20];
    for i in 0..(32 * 20) {
        let x = i % 32;
        let y = i / 32;

        if x >= 30 {
            continue;
        }

        let tile_id = (x + y * 30) as u16;
        let palette_entry = example_logo::PALETTE_ASSIGNMENT[tile_id as usize] as u16;
        entries[i] = tile_id | (palette_entry << 12);
    }

    gfx.copy_to_map(20, &entries);

    loop {}
}
