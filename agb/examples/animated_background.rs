#![no_std]
#![no_main]

use agb::{
    display::{
        Priority,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, VRAM_MANAGER},
    },
    include_background_gfx,
};

include_background_gfx!(mod water_tiles, water_tiles => "examples/water_tiles.png");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let vblank = agb::interrupt::VBlank::get();

    let tileset = &water_tiles::water_tiles.tiles;

    VRAM_MANAGER.set_background_palettes(water_tiles::PALETTES);

    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        tileset.format(),
    );

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile((x, y), tileset, water_tiles::water_tiles.tile_settings[0]);
        }
    }

    let mut frame = gfx.frame();
    bg.show(&mut frame);
    frame.commit();

    let mut i = 0;
    loop {
        i = (i + 1) % 8;

        VRAM_MANAGER.replace_tile(tileset, 0, tileset, i);

        vblank.wait_for_vblank();
    }
}
