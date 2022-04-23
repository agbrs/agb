use super::tiled::{RegularMap, TileFormat, TileSet, TileSetting, VRamManager};

crate::include_gfx!("gfx/agb_logo.toml");

pub fn display_logo(map: &mut RegularMap, vram: &mut VRamManager) {
    vram.set_background_palettes(agb_logo::test_logo.palettes);

    let background_tilemap = TileSet::new(agb_logo::test_logo.tiles, TileFormat::FourBpp);

    for y in 0..20 {
        for x in 0..30 {
            let tile_id = y * 30 + x;

            let palette_entry = agb_logo::test_logo.palette_assignments[tile_id as usize];
            let tile_setting = TileSetting::new(tile_id, false, false, palette_entry);

            map.set_tile(vram, (x, y).into(), &background_tilemap, tile_setting);
        }
    }

    map.commit(vram);
    map.show();
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn logo_display(gba: &mut crate::Gba) {
        let (gfx, mut vram) = gba.display.video.tiled0();

        let mut map = gfx.background(crate::display::Priority::P0);

        display_logo(&mut map, &mut vram);

        crate::test_runner::assert_image_output("gfx/test_logo.png");
    }
}
