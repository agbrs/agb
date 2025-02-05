use agb::{
    display::{
        tiled::{
            RegularBackgroundSize, RegularBackgroundTiles, TileSet, TileSetting, VRAM_MANAGER,
        },
        GraphicsFrame, Priority,
    },
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

pub fn load_palettes() {
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);
}

pub(crate) fn load_help_text(
    background: &mut RegularBackgroundTiles,
    help_text_line: u16,
    at_tile: (u16, u16),
) {
    let help_tiledata = &backgrounds::help;

    for x in 0..16 {
        let tile_id = help_text_line * 16 + x;

        background.set_tile(
            (x + at_tile.0, at_tile.1),
            &help_tiledata.tiles,
            help_tiledata.tile_settings[tile_id as usize],
        )
    }
}

pub(crate) fn load_description(face_id: usize, descriptions_map: &mut RegularBackgroundTiles) {
    let description_data = if face_id < 10 {
        &backgrounds::descriptions1
    } else {
        &backgrounds::descriptions2
    };

    for y in 0..11 {
        for x in 0..8 {
            let tile_id = y * 8 + x + 8 * 11 * (face_id as u16 % 10);
            descriptions_map.set_tile(
                (x, y),
                &description_data.tiles,
                description_data.tile_settings[tile_id as usize],
            )
        }
    }
}

// Expects a 64x32 map
fn create_background_map(stars_tileset: &TileSet) -> RegularBackgroundTiles {
    let mut map = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background64x32,
        stars_tileset.format(),
    );

    for x in 0..64u16 {
        for y in 0..32u16 {
            let blank = rng::gen().rem_euclid(32) < 30;

            let tile_setting = if blank {
                TileSetting::BLANK
            } else {
                let tile_id = rng::gen().rem_euclid(64) as u16;
                backgrounds::stars.tile_settings[tile_id as usize]
            };

            map.set_tile((x, y), stars_tileset, tile_setting);
        }
    }

    map.set_scroll_pos((0i16, rng::gen().rem_euclid(8) as i16));

    map
}

pub fn show_title_screen(sfx: &mut Sfx) -> RegularBackgroundTiles {
    let mut background = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        backgrounds::title.tiles.format(),
    );
    background.set_scroll_pos((0i16, 0));
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    background.fill_with(&backgrounds::title);
    sfx.frame();

    background.commit();

    background
}

pub struct StarBackground {
    background1: RegularBackgroundTiles,
    background2: RegularBackgroundTiles,

    background1_timer: u32,
    background2_timer: u32,
}

impl StarBackground {
    pub fn new() -> Self {
        let background1 = create_background_map(&backgrounds::stars.tiles);
        let background2 = create_background_map(&backgrounds::stars.tiles);

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

    pub fn commit(&mut self) {
        self.background1.commit();
        self.background2.commit();
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.background1.show(frame);
        self.background2.show(frame);
    }
}
