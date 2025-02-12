use agb::display::{
    tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, TileSet, TileSetting},
    GraphicsFrame, Priority, HEIGHT, WIDTH,
};

const LEVEL_START: usize = 12 * 28;
const NUMBERS_START: usize = 12 * 28 + 3;
const HYPHEN: usize = 12 * 28 + 11;
pub const BLANK: usize = 11 * 28;

pub struct LevelDisplay {
    map: RegularBackgroundTiles,
}

impl LevelDisplay {
    pub fn new(tileset: &'_ TileSet<'_>, tile_settings: &[TileSetting]) -> Self {
        let mut map = RegularBackgroundTiles::new(
            Priority::P3,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );

        for y in 0..32 {
            for x in 0..32 {
                map.set_tile((x, y), tileset, tile_settings[BLANK]);
            }
        }

        map.set_scroll_pos((-(WIDTH / 2 - 7 * 8 / 2) as i16, -(HEIGHT / 2 - 4) as i16));

        Self { map }
    }

    pub fn write_level(
        &mut self,
        tileset: &'_ TileSet<'_>,
        tile_settings: &[TileSetting],
        world: u32,
        level: u32,
    ) {
        for (i, &tile) in [
            LEVEL_START,
            LEVEL_START + 1,
            LEVEL_START + 2,
            BLANK,
            world as usize + NUMBERS_START - 1,
            HYPHEN,
            level as usize + NUMBERS_START - 1,
        ]
        .iter()
        .enumerate()
        {
            self.map
                .set_tile((i as i32, 0), tileset, tile_settings[tile]);
        }

        self.map.commit();
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.map.show(frame);
    }
}
