//! This example demonstrates sprite rendering order. Sprites drawn before other sprites
//! will show up above those drawn afterwards.
#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    Gba,
    display::{
        Priority,
        object::Object,
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
    },
    include_aseprite, include_background_gfx,
    input::{Button, ButtonController},
};

use agb_fixnum::vec2;
use alloc::vec::Vec;

include_aseprite!(mod sprites, "examples/gfx/number_sprites.aseprite");
include_background_gfx!(mod bg, BG => "examples/gfx/object_z_order_background.aseprite");

#[agb::entry]
fn entry(mut gba: Gba) -> ! {
    let mut gfx = gba.graphics.get();
    VRAM_MANAGER.set_background_palettes(bg::PALETTES);

    let mut background = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    background.fill_with(&bg::BG);

    let sprites = (0..4)
        .map(|i| (i, sprites::NUMBERS.animation_sprite(i)))
        .collect::<Vec<_>>();

    let mut first_to_draw = 0;

    let mut input = ButtonController::new();

    loop {
        if input.is_just_pressed(Button::A) {
            first_to_draw = (first_to_draw + 1) % sprites.len();
        }

        let mut frame = gfx.frame();

        let (front, back) = sprites.split_at(first_to_draw);

        for &(i, sprite) in back.iter().chain(front) {
            Object::new(sprite)
                .set_pos(vec2(81, 72) + vec2(4, 3) * i as i32)
                .show(&mut frame);
        }

        // show the render order
        for (i, &(_sprite_number, sprite)) in back.iter().chain(front).enumerate() {
            Object::new(sprite)
                .set_pos(vec2(59, 18) + vec2(13 * i as i32, 0))
                .show(&mut frame);
        }

        background.show(&mut frame);
        frame.commit();
        input.update();
    }
}
