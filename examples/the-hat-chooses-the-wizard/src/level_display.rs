use agb::display::{background::BackgroundRegister, HEIGHT, WIDTH};

const LEVEL_START: u16 = 12 * 28;
const NUMBERS_START: u16 = 12 * 28 + 3;
const HYPHEN: u16 = 12 * 28 + 11;
pub const BLANK: u16 = 11 * 28;

pub fn write_level(background: &mut BackgroundRegister, world: u32, level: u32) {
    let map = background.get_block();
    let mut counter = 0;

    map[0][0] = LEVEL_START;
    map[0][1] = LEVEL_START + 1;
    map[0][2] = LEVEL_START + 2;

    counter += 4;

    map[0][counter] = world as u16 + NUMBERS_START - 1;
    counter += 1;
    map[0][counter] = HYPHEN;
    counter += 1;
    map[0][counter] = level as u16 + NUMBERS_START - 1;
    counter += 1;

    background.set_position((-(WIDTH / 2 - counter as i32 * 8 / 2), -(HEIGHT / 2 - 4)).into());
}
