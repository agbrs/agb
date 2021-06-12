#![no_std]
#![no_main]

extern crate agb;

use agb::display::example_logo;

#[no_mangle]
pub fn main() -> ! {
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

    back.draw_full_map(&entries, (30_u32, 20_u32).into(), 0);
    back.show();

    loop {}
}
