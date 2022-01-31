use agb::display::{
    background::{RegularMap, TileSetReference, TileSetting, VRamManager},
    HEIGHT, WIDTH,
};

const LEVEL_START: u16 = 12 * 28;
const NUMBERS_START: u16 = 12 * 28 + 3;
const HYPHEN: u16 = 12 * 28 + 11;
pub const BLANK: u16 = 11 * 28;

pub fn write_level(
    map: &mut RegularMap,
    world: u32,
    level: u32,
    tile_set_ref: TileSetReference,
    vram: &mut VRamManager,
) {
    for (i, &tile) in [
        LEVEL_START,
        LEVEL_START + 1,
        LEVEL_START + 2,
        BLANK,
        world as u16 + NUMBERS_START - 1,
        HYPHEN,
        level as u16 + NUMBERS_START - 1,
    ]
    .iter()
    .enumerate()
    {
        map.set_tile(
            vram,
            (i as u16, 0).into(),
            tile_set_ref,
            TileSetting::from_raw(tile),
        );
    }

    map.set_scroll_pos(
        (
            -(WIDTH / 2 - 7 as i32 * 8 / 2) as u16,
            -(HEIGHT / 2 - 4) as u16,
        )
            .into(),
    );
}
