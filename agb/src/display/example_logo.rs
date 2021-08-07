use crate::display::tiled0::Tiled0;

crate::include_gfx!("gfx/agb_logo.toml");

use super::tiled0::Map;

pub fn display_logo(gfx: &mut Tiled0) {
    gfx.set_background_palettes(agb_logo::test_logo.palettes);
    gfx.set_background_tilemap(0, agb_logo::test_logo.tiles);

    let mut back = gfx.get_background().unwrap();

    let mut entries: [u16; 30 * 20] = [0; 30 * 20];
    for tile_id in 0..(30 * 20) {
        let palette_entry = agb_logo::test_logo.palette_assignments[tile_id as usize] as u16;
        entries[tile_id as usize] = tile_id | (palette_entry << 12);
    }

    back.set_map(Map {
        store: entries.as_ref(),
        dimensions: (30_u32, 20_u32).into(),
        default: 0,
    });
    back.show();
}

#[test_case]
fn logo_display(gba: &mut crate::Gba) {
    let mut gfx = gba.display.video.tiled0();

    display_logo(&mut gfx);

    crate::assert_image_output("gfx/test_logo.png");
}
