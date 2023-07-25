use agb::{
    display::tiled::{RegularMap, TileFormat, TileSet, TileSetting, VRamManager},
    include_background_gfx,
};

include_background_gfx!(backgrounds, "1e151b",
    ui => "maps/ui_tiles.png",
    level => "maps/level.png",
    ending => "gfx/ending_page.aseprite",
);

mod tilemaps {
    use super::backgrounds;
    include!(concat!(env!("OUT_DIR"), "/tilemaps.rs"));
}

pub fn load_palettes(vram_manager: &mut VRamManager) {
    vram_manager.set_background_palettes(backgrounds::PALETTES);
}

pub fn load_ui(map: &mut RegularMap, vram_manager: &mut VRamManager) {
    let ui_tileset = TileSet::new(backgrounds::ui.tiles, TileFormat::FourBpp);

    for y in 0..20u16 {
        for x in 0..30u16 {
            let tile_pos = y * 30 + x;
            let tile_setting = tilemaps::UI_BACKGROUND_MAP[tile_pos as usize];

            map.set_tile(vram_manager, (x, y).into(), &ui_tileset, tile_setting);
        }
    }
}

pub fn load_level_background(
    map: &mut RegularMap,
    vram_manager: &mut VRamManager,
    level_number: usize,
) {
    let level_map = &tilemaps::LEVELS_MAP[level_number];

    let level_tileset = TileSet::new(backgrounds::level.tiles, TileFormat::FourBpp);

    for y in 0..20u16 {
        for x in 0..22u16 {
            let tile_pos = y * 22 + x;
            let tile_setting = level_map[tile_pos as usize];

            map.set_tile(vram_manager, (x, y).into(), &level_tileset, tile_setting);
        }
    }
}

pub fn load_ending_page(map: &mut RegularMap, vram_manager: &mut VRamManager) {
    let ending_tileset = TileSet::new(backgrounds::ending.tiles, TileFormat::FourBpp);

    for y in 0..20u16 {
        for x in 0..30u16 {
            let tile_pos = y * 30 + x;
            let tile_setting = TileSetting::new(
                tile_pos,
                false,
                false,
                backgrounds::ending.palette_assignments[tile_pos as usize],
            );
            map.set_tile(vram_manager, (x, y).into(), &ending_tileset, tile_setting);
        }
    }
}
