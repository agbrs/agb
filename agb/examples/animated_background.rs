#![no_std]
#![no_main]

use core::cmp::Ordering;

use agb::{
    display::{
        Priority,
        tiled::{
            RegularBackgroundSize, RegularBackgroundTiles, TileEffect, TileSetting, VRAM_MANAGER,
        },
    },
    include_background_gfx,
};

// explicitly not using `deduplicate` here because we want the tile IDs to stay consistent
include_background_gfx!(mod background, platformer => "examples/gfx/platformer-background.aseprite");

mod background_tile_ids {
    use core::ops::Range;

    pub const GRASS: u16 = 0;
    pub const SKY: u16 = 1;
    pub const GROUND: u16 = 2;
    pub const SUNFLOWER: Range<u16> = 3..6;
    pub const SUNFLOWER_STEM: u16 = 6;
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let vblank = agb::interrupt::VBlank::get();

    let tileset = &background::platformer.tiles;

    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let mut bg = RegularBackgroundTiles::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        tileset.format(),
    );

    for y in 0..20u16 {
        for x in 0..30u16 {
            let tile_index = match y.cmp(&10) {
                Ordering::Less => background_tile_ids::SKY,
                Ordering::Equal => background_tile_ids::GRASS,
                Ordering::Greater => background_tile_ids::GROUND,
            };

            bg.set_tile(
                (x, y),
                tileset,
                TileSetting::new(tile_index, TileEffect::default()),
            );
        }
    }

    let mut foreground = RegularBackgroundTiles::new(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        tileset.format(),
    );

    for x in 0..30u16 {
        if x % 3 == 0 {
            // put a sunflower here. Stem first
            foreground.set_tile(
                (x, 9),
                tileset,
                TileSetting::new(background_tile_ids::SUNFLOWER_STEM, TileEffect::default()),
            );
            // now the flower head
            foreground.set_tile(
                (x, 8),
                tileset,
                TileSetting::new(background_tile_ids::SUNFLOWER.start, TileEffect::default()),
            );
        }
    }

    let mut frame = gfx.frame();
    bg.show(&mut frame);
    foreground.show(&mut frame);
    frame.commit();

    let mut frame_skip = 0;
    let mut sunflower_frame = background_tile_ids::SUNFLOWER
        .chain(background_tile_ids::SUNFLOWER.rev())
        .cycle();
    loop {
        if frame_skip == 0 {
            // note that we're not even showing the frames again, we're replacing the tile data
            // in video RAM which will show up when that frame needs to be shown
            VRAM_MANAGER.replace_tile(
                tileset,
                background_tile_ids::SUNFLOWER.start,
                tileset,
                sunflower_frame.next().unwrap(),
            );
            frame_skip = 30;
        } else {
            frame_skip -= 1;
        }

        vblank.wait_for_vblank();
    }
}
