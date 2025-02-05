use agb::{
    display::tiled::{RegularBackgroundTiles, VRAM_MANAGER},
    include_background_gfx,
};

include_background_gfx!(backgrounds, "1e151b",
    ui => deduplicate "maps/ui_tiles.png",
    level => deduplicate "maps/level.png",
    ending => deduplicate "gfx/ending_page.aseprite",
);

mod tilemaps {
    use super::backgrounds;
    include!(concat!(env!("OUT_DIR"), "/tilemaps.rs"));
}

pub fn load_palettes() {
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);
}

pub fn load_ui(map: &mut RegularBackgroundTiles) {
    let ui_tileset = &backgrounds::ui.tiles;

    for y in 0..20u16 {
        for x in 0..30u16 {
            let tile_pos = y * 30 + x;
            let tile_setting = tilemaps::UI_BACKGROUND_MAP[tile_pos as usize];

            map.set_tile((x, y), ui_tileset, tile_setting);
        }
    }
}

pub fn load_level_background(map: &mut RegularBackgroundTiles, level_number: usize) {
    let level_map = &tilemaps::LEVELS_MAP[level_number];

    let level_tileset = &backgrounds::level.tiles;

    for y in 0..20u16 {
        for x in 0..22u16 {
            let tile_pos = y * 22 + x;
            let tile_setting = level_map[tile_pos as usize];

            map.set_tile((x, y), level_tileset, tile_setting);
        }
    }
}

pub fn load_ending_page(map: &mut RegularBackgroundTiles) {
    map.fill_with(&backgrounds::ending);
}
