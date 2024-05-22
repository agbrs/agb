use super::tiled::{RegularBackgroundTiles, VRamManager};

crate::include_background_gfx!(crate, agb_logo, test_logo => deduplicate "gfx/test_logo.png");
crate::include_background_gfx!(crate, agb_logo_basic, test_logo => deduplicate "gfx/test_logo_basic.png");

pub fn display_logo(map: &mut RegularBackgroundTiles, vram: &mut VRamManager) {
    vram.set_background_palettes(agb_logo::PALETTES);

    map.fill_with(vram, &agb_logo::test_logo);
}

pub fn display_logo_basic(map: &mut RegularMap, vram: &mut VRamManager) {
    vram.set_background_palettes(agb_logo_basic::PALETTES);

    map.fill_with(vram, &agb_logo_basic::test_logo);

    map.commit(vram);
    map.set_visible(true);
}

#[cfg(test)]
mod tests {
    use crate::display::{
        tiled::{RegularBackgroundSize, RegularBackgroundTiles},
        Priority,
    };

    use super::*;

    #[test_case]
    fn logo_display(gba: &mut crate::Gba) {
        let (mut gfx, mut vram) = gba.display.video.tiled();

        let mut map = RegularBackgroundTiles::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            agb_logo::test_logo.tiles.format(),
        );

        display_logo(&mut map, &mut vram);
        map.commit();

        let mut bg_iter = gfx.iter();
        map.show(&mut bg_iter);
        bg_iter.commit(&mut vram);

        crate::test_runner::assert_image_output("gfx/test_logo.png");

        map.clear(&mut vram);
    }
}
