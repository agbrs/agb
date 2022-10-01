use agb::{
    display::tiled::{RegularMap, TileFormat, TileSet, TileSetting, TiledMap, VRamManager},
    include_gfx, rng,
};

use crate::sfx::Sfx;

include_gfx!("gfx/backgrounds.toml");

pub fn load_palettes(vram: &mut VRamManager) {
    vram.set_background_palettes(&[
        backgrounds::stars.palettes[0].clone(),
        backgrounds::descriptions1.palettes[0].clone(),
        backgrounds::descriptions2.palettes[0].clone(),
        backgrounds::help.palettes[0].clone(),
    ]);
}

pub(crate) fn load_help_text(
    vram: &mut VRamManager,
    background: &mut RegularMap,
    help_text_line: u16,
    at_tile: (u16, u16),
) {
    let help_tileset = TileSet::new(
        backgrounds::help.tiles,
        agb::display::tiled::TileFormat::FourBpp,
    );

    for x in 0..16 {
        background.set_tile(
            vram,
            (x + at_tile.0, at_tile.1).into(),
            &help_tileset,
            TileSetting::new(help_text_line * 16 + x, false, false, 3),
        )
    }
}

pub(crate) fn load_description(
    face_id: usize,
    descriptions_map: &mut RegularMap,
    vram: &mut VRamManager,
) {
    let tileset = if face_id < 10 {
        TileSet::new(
            backgrounds::descriptions1.tiles,
            agb::display::tiled::TileFormat::FourBpp,
        )
    } else {
        TileSet::new(
            backgrounds::descriptions2.tiles,
            agb::display::tiled::TileFormat::FourBpp,
        )
    };

    for y in 0..11 {
        for x in 0..8 {
            descriptions_map.set_tile(
                vram,
                (x, y).into(),
                &tileset,
                TileSetting::new(y * 8 + x + 8 * 11 * (face_id as u16 - 10), false, false, 2),
            )
        }
    }
}

// Expects a 64x32 map
fn create_background_map(map: &mut RegularMap, vram: &mut VRamManager, stars_tileset: &TileSet) {
    for x in 0..64u16 {
        for y in 0..32u16 {
            let blank = rng::gen().rem_euclid(32) < 30;

            let tile_id = if blank {
                (1 << 10) - 1
            } else {
                rng::gen().rem_euclid(64) as u16
            };
            let tile_setting = TileSetting::new(tile_id, false, false, 0);

            map.set_tile(vram, (x, y).into(), stars_tileset, tile_setting);
        }
    }

    map.set_scroll_pos((0i16, rng::gen().rem_euclid(8) as i16).into());
}

pub fn show_title_screen(background: &mut RegularMap, vram: &mut VRamManager, sfx: &mut Sfx) {
    background.set_scroll_pos((0i16, 0).into());
    vram.set_background_palettes(backgrounds::title.palettes);
    let tile_set = TileSet::new(
        backgrounds::title.tiles,
        agb::display::tiled::TileFormat::FourBpp,
    );
    background.hide();

    for x in 0..30u16 {
        for y in 0..20u16 {
            let tile_id = y * 30 + x;
            background.set_tile(
                vram,
                (x, y).into(),
                &tile_set,
                TileSetting::new(
                    tile_id,
                    false,
                    false,
                    backgrounds::title.palette_assignments[tile_id as usize],
                ),
            );
        }

        sfx.frame();
    }

    background.commit(vram);
    sfx.frame();
    background.show();
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
        let stars_tileset = TileSet::new(backgrounds::stars.tiles, TileFormat::FourBpp);
        create_background_map(background1, vram, &stars_tileset);
        create_background_map(background2, vram, &stars_tileset);

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

    pub fn hide(&mut self) {
        self.background1.hide();
        self.background2.hide();
    }

    pub fn show(&mut self) {
        self.background1.show();
        self.background2.show();
    }
}
