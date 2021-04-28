#![no_std]
#![feature(start)]

extern crate agb;

use agb::display::example_logo;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = agb::Gba::new();
    let mut gfx = gba.display.video.tiled0();

    gfx.set_background_palettes(example_logo::PALETTE_DATA);
    gfx.set_background_tilemap(0, example_logo::TILE_DATA);

    let mut back = gfx.get_background().unwrap();

    let mut entries: [u16; 30 * 20] = [0; 30 * 20];
    for tile_id in 0..(30 * 20) {
        let palette_entry = example_logo::PALETTE_ASSIGNMENT[tile_id as usize] as u16;
        entries[tile_id as usize] = tile_id | (palette_entry << 12);
    }

    back.set_map(&entries, 30, 20);
    back.set_position(0, 0);
    back.show();

    loop {}
}
