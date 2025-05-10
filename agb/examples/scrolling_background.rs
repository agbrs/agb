//! Shows how backgrounds can be scrolled and how they wrap around.
#![no_std]
#![no_main]

use agb::{
    display::{
        Priority, WIDTH,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    include_background_gfx,
};

include_background_gfx!(mod background,
    WIDE_BACKGROUND => deduplicate "examples/gfx/wide-background.aseprite",
    HELP_TEXT => deduplicate "examples/gfx/wide-background-help-text.aseprite",
);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Set up the background palettes as needed. These are produced by the include_background_gfx! macro call above.
    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    // Get access to the graphics struct which is used to manage the frame lifecycle
    let mut gfx = gba.graphics.get();

    let mut scrolling_tiles = RegularBackgroundTiles::new(
        Priority::P1,
        RegularBackgroundSize::Background64x32,
        TileFormat::FourBpp,
    );

    // the background is 64x20 tiles in size
    for y in 0..20 {
        for x in 0..64 {
            let tile_index = (x + y * 64) as usize;
            scrolling_tiles.set_tile(
                (x, y),
                &background::WIDE_BACKGROUND.tiles,
                background::WIDE_BACKGROUND.tile_settings[tile_index],
            );
        }
    }

    let mut help_tiles = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    // the help tiles are 23x2 tiles in size
    for y in 0..2 {
        for x in 0..23 {
            let tile_index = (x + y * 23) as usize;
            help_tiles.set_tile(
                (x, y),
                &background::HELP_TEXT.tiles,
                background::HELP_TEXT.tile_settings[tile_index],
            );
        }
    }

    // put the help text somewhere near the center
    help_tiles.set_scroll_pos((-(WIDTH - 23 * 8) / 2, -30));

    let mut x_position = 0;

    loop {
        x_position = (x_position + 1) % 512;

        let mut frame = gfx.frame();

        scrolling_tiles.set_scroll_pos((x_position, 0));
        scrolling_tiles.show(&mut frame);

        help_tiles.show(&mut frame);

        frame.commit();
    }
}
