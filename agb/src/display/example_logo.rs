use super::tiled::{RegularBackgroundTiles, VRAM_MANAGER};

crate::include_background_gfx!(crate, agb_logo, test_logo => deduplicate "gfx/test_logo.png");
crate::include_background_gfx!(crate, agb_logo_basic, test_logo => deduplicate "gfx/test_logo_basic.png");

pub fn display_logo(map: &mut RegularBackgroundTiles) {
    VRAM_MANAGER.set_background_palettes(agb_logo::PALETTES);

    map.fill_with(&agb_logo::test_logo);
}

pub fn display_logo_basic(map: &mut RegularBackgroundTiles) {
    VRAM_MANAGER.set_background_palettes(agb_logo_basic::PALETTES);

    map.fill_with(&agb_logo_basic::test_logo);
}

#[cfg(test)]
mod tests {
    use crate::display::{
        Priority,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles},
    };

    use super::*;

    #[test_case]
    fn logo_display(gba: &mut crate::Gba) {
        let mut gfx = gba.graphics.get();

        let mut map = RegularBackgroundTiles::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            agb_logo::test_logo.tiles.format(),
        );

        display_logo(&mut map);

        let mut frame = gfx.frame();
        map.show(&mut frame);
        frame.commit();

        crate::test_runner::assert_image_output("gfx/test_logo.png");
    }
}
