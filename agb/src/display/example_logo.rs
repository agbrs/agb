use super::tiled::{RegularMap, TileFormat, TileSet, TiledMap, VRamManager};

crate::include_background_gfx!(crate, agb_logo, test_logo => deduplicate "gfx/test_logo.png");

pub fn display_logo(map: &mut RegularMap, vram: &mut VRamManager) {
    vram.set_background_palettes(agb_logo::PALETTES);

    let background_tilemap = TileSet::new(agb_logo::test_logo.tiles, TileFormat::FourBpp);

    for y in 0..20 {
        for x in 0..30 {
            let tile_id = y * 30 + x;

            map.set_tile(
                vram,
                (x as u16, y as u16).into(),
                &background_tilemap,
                agb_logo::test_logo.tile_settings[tile_id],
            );
        }
    }

    map.commit(vram);
    map.show();
}

#[cfg(test)]
mod tests {
    use crate::display::{tiled::RegularBackgroundSize, Priority};

    use super::*;

    #[test_case]
    fn logo_display(gba: &mut crate::Gba) {
        let (gfx, mut vram) = gba.display.video.tiled0();

        let mut map = gfx.background(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );

        display_logo(&mut map, &mut vram);

        crate::test_runner::assert_image_output("gfx/test_logo.png");

        map.clear(&mut vram);
        vram.gc();
    }
}
