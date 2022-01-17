use crate::display::background::Tiled0;

use super::background::{Tile, TileFormat, TileSet};

crate::include_gfx!("gfx/agb_logo.toml");

pub fn display_logo(gfx: &mut Tiled0) {
    gfx.vram
        .set_background_palettes(agb_logo::test_logo.palettes);

    let background_tilemap = TileSet::new(agb_logo::test_logo.tiles, TileFormat::FourBpp);
    let background_tilemap_reference = gfx.vram.add_tileset(background_tilemap);

    let mut back = gfx.background();

    for y in 0..20 {
        for x in 0..30 {
            let tile_id = y * 30 + x;

            let palette_entry = agb_logo::test_logo.palette_assignments[tile_id as usize] as u16;
            let tile = gfx.vram.add_tile(background_tilemap_reference, tile_id);

            back.set_tile(x, y, Tile::new(tile, false, false, palette_entry))
        }
    }

    back.commit();
    back.show();
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn logo_display(gba: &mut crate::Gba) {
        let mut gfx = gba.display.video.tiled0();

        display_logo(&mut gfx);

        crate::test_runner::assert_image_output("gfx/test_logo.png");
    }
}
