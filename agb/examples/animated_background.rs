#![no_std]
#![no_main]

use agb::{
    display::{
        tiled::{RegularBackgroundSize, RegularBackgroundTiles},
        Priority,
    },
    include_background_gfx,
};

include_background_gfx!(water_tiles, water_tiles => "examples/water_tiles.png");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (mut gfx, mut vram) = gba.display.video.tiled();
    let vblank = agb::interrupt::VBlank::get();

    let tileset = &water_tiles::water_tiles.tiles;

    vram.set_background_palettes(water_tiles::PALETTES);

    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        tileset.format(),
    );

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile(
                &mut vram,
                (x, y),
                tileset,
                water_tiles::water_tiles.tile_settings[0],
            );
        }
    }

    bg.commit();

    let mut bg_iter = gfx.iter();
    bg.show(&mut bg_iter);
    bg_iter.commit(&mut vram);

    let mut i = 0;
    loop {
        i = (i + 1) % 8;

        vram.replace_tile(tileset, 0, tileset, i);

        vblank.wait_for_vblank();
    }
}
