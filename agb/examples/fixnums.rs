#![no_std]
#![no_main]

use agb::{
    display::{
        Graphics, Rgb15,
        object::{Object, SpriteVram},
        tiled::VRAM_MANAGER,
    },
    fixnum::{Num, Vector2D, vec2},
    include_aseprite,
    input::{Button, ButtonController},
};

include_aseprite!(mod sprites, "examples/gfx/crab.aseprite");

fn integer(
    gfx: &mut Graphics,
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
            .set_position(position)
            .show(&mut frame);

        frame.commit();
        button.update();
    }

    (position, velocity)
}

fn fixed(
    gfx: &mut Graphics,
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
            .set_position(position.floor())
            .show(&mut frame);

        frame.commit();
        button.update();
    }

    (position.floor(), velocity.floor())
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb15::WHITE);

    let mut position = vec2(80, 80);
    let mut velocity = vec2(0, 0);
    loop {
        (position, velocity) = integer(&mut gfx, position, velocity);
        (position, velocity) = fixed(&mut gfx, position, velocity);
    }
}
