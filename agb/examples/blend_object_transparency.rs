//! An example of using blending to make an object transparent. Only the objects marked with
//! GraphicsMode::AlphaBlending will show as transparent.
#![no_std]
#![no_main]

use agb::{
    display::{
        Priority,
        object::{GraphicsMode, Object},
        tiled::{RegularBackground, RegularBackgroundSize, VRAM_MANAGER},
    },
    fixnum::{Num, num},
    include_aseprite, include_background_gfx,
};

include_aseprite!(mod sprites, "examples/gfx/crab.aseprite");
include_background_gfx!(mod background, BEACH => deduplicate "examples/gfx/beach-background.aseprite");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let mut gfx = gba.graphics.get();

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        background::BEACH.tiles.format(),
    );
    bg.fill_with(&background::BEACH);

    let mut transparency_amount = num!(0);

    loop {
        let mut frame = gfx.frame();
        let bg_id = bg.show(&mut frame);

        Object::new(sprites::IDLE.sprite(0))
            .set_pos((100, 100))
            .show(&mut frame);

        Object::new(sprites::IDLE.sprite(0))
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .set_pos((150, 100))
            .show(&mut frame);

        frame
            .blend()
            .object_transparency(transparency_amount, num!(1) - transparency_amount)
            .enable_background(bg_id);
        transparency_amount = (transparency_amount + Num::from_raw(1)) % num!(1);

        frame.commit();
    }
}
