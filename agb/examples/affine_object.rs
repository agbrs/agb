//! This example shows how you can use affine objects to rotate and scale. It also shows
//! the importance of the `AffineDouble` display mode.
#![no_std]
#![no_main]

use agb::{
    display::{
        GraphicsFrame, HEIGHT, Rgb15, WIDTH,
        affine::AffineMatrix,
        object::{AffineMatrixInstance, AffineMode, Object, ObjectAffine, SpriteVram},
        tiled::VRAM_MANAGER,
    },
    fixnum::{Num, num, vec2},
    include_aseprite,
};

include_aseprite!(mod sprites,
    "examples/gfx/crab.aseprite",
    "examples/gfx/box.aseprite",
);

fn show_with_boxes(matrix: AffineMatrix, height: i32, frame: &mut GraphicsFrame) {
    /// Calculate the x coordinate for a crab with a given index
    fn x(idx: i32) -> i32 {
        (WIDTH / 4) * (idx + 1) - 16
    }

    let crab = SpriteVram::from(sprites::IDLE.sprite(0));
    let square = SpriteVram::from(sprites::BOX.sprite(0));
    let instance = AffineMatrixInstance::new(matrix.to_object_wrapping());

    for idx in 0..3 {
        Object::new(square.clone())
            .set_pos((x(idx), height))
            .show(frame);
    }

    Object::new(crab.clone())
        .set_pos((x(0), height))
        .show(frame);
    ObjectAffine::new(crab.clone(), instance.clone(), AffineMode::Affine)
        .set_pos((x(1), height))
        .show(frame);
    ObjectAffine::new(crab.clone(), instance.clone(), AffineMode::AffineDouble)
        .set_pos((x(2), height))
        .show(frame);
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb15::WHITE);

    let mut angle: Num<i32, 8> = num!(0);

    loop {
        let mut frame = gfx.frame();

        angle += num!(1. / 64.);
        angle %= num!(1.);

        let rotation_matrix = AffineMatrix::from_rotation(angle);
        let scale_matrix = AffineMatrix::from_scale(vec2(num!(0.5), num!(0.5)));

        show_with_boxes(rotation_matrix, HEIGHT / 3 - 16, &mut frame);

        show_with_boxes(
            rotation_matrix * scale_matrix,
            HEIGHT / 3 * 2 - 16,
            &mut frame,
        );

        frame.commit();
    }
}
