//! How to use the infinite scrolled map to make a background that's bigger than the maximum background
//! size on the game boy advance. Creates a wrapping map which you can look around.
#![no_std]
#![no_main]

use agb::{
    display::{
        Priority,
        tiled::{InfiniteScrolledMap, RegularBackground, RegularBackgroundSize, VRAM_MANAGER},
    },
    fixnum::vec2,
    include_background_gfx,
    input::ButtonController,
};

include_background_gfx!(mod big_map, "2ce8f4", big_map => deduplicate "examples/big_map.png");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let mut input = ButtonController::new();

    let tileset = &big_map::big_map.tiles;

    VRAM_MANAGER.set_background_palettes(big_map::PALETTES);

    let bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        tileset.format(),
    );

    let mut infinite_scrolled = InfiniteScrolledMap::new(bg);
    let mut current_pos = vec2(0, 0);

    loop {
        input.update();

        current_pos += input.vector();

        infinite_scrolled.set_scroll_pos(current_pos, |p| {
            // The map is 60x40 tiles in size. And we use `rem_euclid` because that will effectively do
            // what % does but will always return a positive result, being exactly what we need for this
            // wrapped map.
            let tile_index = p.x.rem_euclid(big_map::big_map.width as i32) as usize
                + p.y.rem_euclid(big_map::big_map.height as i32) as usize * 60;

            (
                &big_map::big_map.tiles,
                big_map::big_map.tile_settings[tile_index],
            )
        });

        let mut frame = gfx.frame();
        infinite_scrolled.show(&mut frame);

        frame.commit();
    }
}
