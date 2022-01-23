use super::background::{RegularMap, TileFormat, TileSet, TileSetting, VRamManager};

crate::include_gfx!("gfx/agb_logo.toml");

pub fn display_logo(map: &mut RegularMap, vram: &mut VRamManager) {
    vram.set_background_palettes(agb_logo::test_logo.palettes);

    let background_tilemap = TileSet::new(agb_logo::test_logo.tiles, TileFormat::FourBpp);
    let background_tilemap_reference = vram.add_tileset(background_tilemap);

    for y in 0..20 {
        for x in 0..30 {
            let tile_id = y * 30 + x;

            let palette_entry = agb_logo::test_logo.palette_assignments[tile_id as usize];
            let tile_setting = TileSetting::new(false, false, palette_entry);

            map.set_tile(
                vram,
                (x, y).into(),
                background_tilemap_reference,
                tile_id,
                tile_setting,
            );
        }
    }

    map.commit();
    map.show();
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn logo_display(gba: &mut crate::Gba) {
        let mut gfx = gba.display.video.tiled0();

        let mut map = gfx.background();

        display_logo(&mut map, &mut gfx.vram);

        crate::test_runner::assert_image_output("gfx/test_logo.png");
    }
}
