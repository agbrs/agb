use agb::display::{
    tiled::{RegularMap, TileSet, TileSetting, VRamManager},
    HEIGHT, WIDTH,
};

const LEVEL_START: usize = 12 * 28;
const NUMBERS_START: usize = 12 * 28 + 3;
const HYPHEN: usize = 12 * 28 + 11;
pub const BLANK: usize = 11 * 28;

pub fn write_level(
    map: &mut RegularMap,
    world: u32,
    level: u32,
    tileset: &'_ TileSet<'_>,
    vram: &mut VRamManager,
    tile_settings: &[TileSetting],
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
        map.set_tile(vram, (i as u16, 0).into(), tileset, tile_settings[tile]);
    }

    map.set_scroll_pos((-(WIDTH / 2 - 7 * 8 / 2) as i16, -(HEIGHT / 2 - 4) as i16).into());
}
