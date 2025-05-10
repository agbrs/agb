//! Shows how you can create smoother motion using fixnums rather than integer coordinates.
//!
//! When switching between integer and fixnum versions, the code is identical between them
//! (applying some acceleration) but it is impossible to produce acceleration slow enough with integers
#![no_std]
#![no_main]

use agb::{
    display::{
        Graphics,
        object::{Object, SpriteVram},
        tiled::{RegularBackgroundTiles, VRAM_MANAGER},
    },
    fixnum::{Num, Vector2D, vec2},
    include_aseprite, include_background_gfx,
    input::{Button, ButtonController},
};

include_aseprite!(mod sprites, "examples/gfx/crab.aseprite");
include_background_gfx!(mod background, bg => deduplicate "examples/gfx/fixed_point_background.aseprite");

fn integer(
    gfx: &mut Graphics,
    bg: &RegularBackgroundTiles,
    initial_position: Vector2D<i32>,
    initial_velocity: Vector2D<i32>,
) -> (Vector2D<i32>, Vector2D<i32>) {
    let mut position = initial_position;
    let mut velocity = initial_velocity;

    let mut button = ButtonController::new();

    let sprite = SpriteVram::from(sprites::IDLE.sprite(0));

    while !button.is_just_pressed(Button::A) {
        velocity *= 7;
        velocity /= 8;
        velocity += button.vector();

        position += velocity;

        let mut frame = gfx.frame();

        Object::new(sprite.clone())
            .set_pos(position)
            .show(&mut frame);

        bg.show(&mut frame);

        frame.commit();
        button.update();
    }

    (position, velocity)
}

fn fixed(
    gfx: &mut Graphics,
    bg: &RegularBackgroundTiles,
    initial_position: Vector2D<i32>,
    initial_velocity: Vector2D<i32>,
) -> (Vector2D<i32>, Vector2D<i32>) {
    let mut position: Vector2D<Num<i32, 8>> = initial_position.change_base();
    let mut velocity: Vector2D<Num<i32, 8>> = initial_velocity.change_base();

    let mut button = ButtonController::new();

    let sprite = SpriteVram::from(sprites::IDLE.sprite(0));

    while !button.is_just_pressed(Button::A) {
        velocity *= 7;
        velocity /= 8;
        velocity += button.vector();

        position += velocity;

        let mut frame = gfx.frame();

        Object::new(sprite.clone())
            .set_pos(position.floor())
            .show(&mut frame);

        bg.show(&mut frame);

        frame.commit();
        button.update();
    }

    (position.floor(), velocity.floor())
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let mut bg = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        agb::display::tiled::RegularBackgroundSize::Background32x32,
        agb::display::tiled::TileFormat::FourBpp,
    );

    VRAM_MANAGER.set_background_palettes(background::PALETTES);
    bg.fill_with(&background::bg);

    let mut position = vec2(80, 80);
    let mut velocity = vec2(0, 0);
    loop {
        (position, velocity) = integer(&mut gfx, &bg, position, velocity);
        (position, velocity) = fixed(&mut gfx, &bg, position, velocity);
    }
}
