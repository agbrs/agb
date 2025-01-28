use agb::display::{
    tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, TileSet, TileSetting},
    Priority, HEIGHT, WIDTH,
};

const LEVEL_START: usize = 12 * 28;
const NUMBERS_START: usize = 12 * 28 + 3;
const HYPHEN: usize = 12 * 28 + 11;
pub const BLANK: usize = 11 * 28;

pub fn write_level(
    world: u32,
    level: u32,
    tileset: &'_ TileSet<'_>,
    tile_settings: &[TileSetting],
) -> RegularBackgroundTiles {
    let mut map = RegularBackgroundTiles::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

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
        map.set_tile((i as i32, 0), tileset, tile_settings[tile]);
    }

    map.set_scroll_pos((-(WIDTH / 2 - 7 * 8 / 2) as i16, -(HEIGHT / 2 - 4) as i16));

    map
}
