//! In this example, we show an affine background used to create a scrollable area (using the D-Pad)
//! which rotates and zooms slightly as you move around.
#![no_std]
#![no_main]

use agb::{
    display::{
        HEIGHT, Priority, WIDTH,
        tiled::{
            AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour,
            AffineMatrixBackground, VRAM_MANAGER,
        },
    },
    fixnum::{Num, Vector2D, num, vec2},
    include_background_gfx,
};

// Import the background. This must be in 256 colour mode because that is the tile format
// required for affine backgrounds.
include_background_gfx!(mod backgrounds,
    "000000",
    NUMBERS => 256 "examples/gfx/number-background.aseprite",
);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    let mut bg = create_background();

    let mut input = agb::input::ButtonController::new();
    let mut position: Vector2D<Num<i32, 8>> = vec2(num!(0), num!(0));

    let mut rotation = num!(0);
    let mut zoom: Num<i32, 16> = num!(1);

    loop {
        input.update();

        position += input.vector();

        // linearly interpolate towards the target rotation for smooth motion
        let target_rotation = match input.x_tri() {
            agb::input::Tri::Positive => num!(-0.01),
            agb::input::Tri::Zero => num!(0.0),
            agb::input::Tri::Negative => num!(0.01),
        };
        rotation = rotation * num!(0.9) + target_rotation * num!(0.1);
        if rotation.abs() <= num!(0.0005) {
            // because of precision issues, we need to make this check since we'll never actually reach 0 otherwise
            rotation = num!(0);
        }

        let target_zoom = if input.vector() == vec2(0, 0) {
            num!(1)
        } else {
            num!(1.2)
        };
        zoom = zoom * num!(0.9) + target_zoom * num!(0.1);

        let transformation = AffineMatrixBackground::from_scale_rotation_position(
            position + vec2(num!(WIDTH), num!(HEIGHT)) / 2,
            (zoom.change_base(), zoom.change_base()),
            rotation,
            -vec2(position.x.round() as i16, position.y.round() as i16)
                + vec2(WIDTH as i16, HEIGHT as i16) / 2,
        );

        bg.set_transform(transformation);

        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.commit();
    }
}

fn create_background() -> AffineBackground {
    let mut bg = AffineBackground::new(
        Priority::P0,
        AffineBackgroundSize::Background32x32,
        AffineBackgroundWrapBehaviour::Wrap,
    );

    let tileset = &backgrounds::NUMBERS.tiles;

    // Set up the background tiles. There are 8 numbers and the background is 32 tiles
    // tall so y / 4 will result in 4 repititions of each number.
    for y in 0..32u16 {
        for x in 0..32u16 {
            bg.set_tile((x, y), tileset, y / 4);
        }
    }

    bg
}
