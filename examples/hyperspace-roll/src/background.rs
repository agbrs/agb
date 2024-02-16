use agb::{
    display::tiled::{RegularMap, TileSet, TileSetting, TiledMap, VRamManager},
    include_background_gfx, rng,
};

use crate::sfx::Sfx;

include_background_gfx!(backgrounds, "121105",
    stars => deduplicate "gfx/stars.aseprite",
    title => deduplicate "gfx/title-screen.aseprite",
    help => deduplicate "gfx/help-text.aseprite",
    descriptions1 => deduplicate "gfx/descriptions1.png",
    descriptions2 => deduplicate "gfx/descriptions2.png",
);

pub fn load_palettes(vram: &mut VRamManager) {
    vram.set_background_palettes(backgrounds::PALETTES);
}

pub(crate) fn load_help_text(
    vram: &mut VRamManager,
    background: &mut RegularMap,
    help_text_line: u16,
    at_tile: (u16, u16),
) {
    let help_tiledata = backgrounds::help;

    for x in 0..16 {
        let tile_id = help_text_line * 16 + x;

        background.set_tile(
            vram,
            (x + at_tile.0, at_tile.1).into(),
            &help_tiledata.tiles,
            help_tiledata.tile_settings[tile_id as usize],
        )
    }
}

pub(crate) fn load_description(
    face_id: usize,
    descriptions_map: &mut RegularMap,
    vram: &mut VRamManager,
) {
    let description_data = if face_id < 10 {
        backgrounds::descriptions1
    } else {
        backgrounds::descriptions2
    };

    for y in 0..11 {
        for x in 0..8 {
            let tile_id = y * 8 + x + 8 * 11 * (face_id as u16 % 10);
            descriptions_map.set_tile(
                vram,
                (x, y).into(),
                &description_data.tiles,
                description_data.tile_settings[tile_id as usize],
            )
        }
    }
}

// Expects a 64x32 map
fn create_background_map(map: &mut RegularMap, vram: &mut VRamManager, stars_tileset: &TileSet) {
    for x in 0..64u16 {
        for y in 0..32u16 {
            let blank = rng::gen().rem_euclid(32) < 30;

            let tile_setting = if blank {
                TileSetting::BLANK
            } else {
                let tile_id = rng::gen().rem_euclid(64) as u16;
                backgrounds::stars.tile_settings[tile_id as usize]
            };

            map.set_tile(vram, (x, y).into(), stars_tileset, tile_setting);
        }
    }

    map.set_scroll_pos((0i16, rng::gen().rem_euclid(8) as i16).into());
}

pub fn show_title_screen(background: &mut RegularMap, vram: &mut VRamManager, sfx: &mut Sfx) {
    background.set_scroll_pos((0i16, 0).into());
    vram.set_background_palettes(backgrounds::PALETTES);

    background.set_visible(false);

    background.fill_with(vram, &backgrounds::title);
    background.commit(vram);
    sfx.frame();
    background.set_visible(true);
}

pub struct StarBackground<'a> {
    background1: &'a mut RegularMap,
    background2: &'a mut RegularMap,

    background1_timer: u32,
    background2_timer: u32,
}

impl<'a> StarBackground<'a> {
    pub fn new(
        background1: &'a mut RegularMap,
        background2: &'a mut RegularMap,
        vram: &'_ mut VRamManager,
    ) -> Self {
        create_background_map(background1, vram, &backgrounds::stars.tiles);
        create_background_map(background2, vram, &backgrounds::stars.tiles);

        Self {
            background1,
            background2,

            background1_timer: 0,
            background2_timer: 0,
        }
    }

    pub fn update(&mut self) {
        if self.background1_timer == 0 {
            self.background1
                .set_scroll_pos(self.background1.scroll_pos() + (1i16, 0).into());
            self.background1_timer = 2;
        }

        if self.background2_timer == 0 {
            self.background2
                .set_scroll_pos(self.background2.scroll_pos() + (1i16, 0).into());
            self.background2_timer = 3;
        }

        self.background1_timer -= 1;
        self.background2_timer -= 1;
    }

    pub fn commit(&mut self, vram: &mut VRamManager) {
        self.background1.commit(vram);
        self.background2.commit(vram);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.background1.set_visible(visible);
        self.background2.set_visible(visible);
    }
}
