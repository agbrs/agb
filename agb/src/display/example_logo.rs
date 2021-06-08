use crate::number::Vector2D;

include!(concat!(env!("OUT_DIR"), "/test_logo.rs"));

#[test_case]
fn logo_display(gba: &mut crate::Gba) {
    let mut gfx = gba.display.video.tiled0();

    gfx.set_background_palettes(PALETTE_DATA);
    gfx.set_background_tilemap(0, TILE_DATA);

    let mut back = gfx.get_background().unwrap();

    let mut entries: [u16; 30 * 20] = [0; 30 * 20];
    for tile_id in 0..(30 * 20) {
        let palette_entry = PALETTE_ASSIGNMENT[tile_id as usize] as u16;
        entries[tile_id as usize] = tile_id | (palette_entry << 12);
    }

    back.draw_full_map(&entries, Vector2D::new(30, 20));
    back.show();

    crate::assert_image_output("gfx/test_logo.png");
}
