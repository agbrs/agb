//! Shows how to use DMA with affine backgrounds transformations to display a rotating pipe effect
//! which you can move around in.
#![no_main]
#![no_std]

extern crate alloc;

use agb::{
    display::{
        AffineMatrix, Priority, WIDTH,
        tiled::{
            AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour,
            AffineMatrixBackground, RegularBackground, RegularBackgroundSize, VRAM_MANAGER,
        },
    },
    dma::HBlankDma,
    fixnum::{Num, Vector2D, num, vec2},
    input::{ButtonController, Tri},
};

use alloc::vec::Vec;

agb::include_background_gfx!(mod backgrounds,
    GRID => 256 "examples/gfx/grid-tiles.aseprite",
    HELP => deduplicate "examples/gfx/grid-tiles-help.aseprite",
);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    let bg = grid_background();
    let help = help_background();

    // Calculate the constant part of the scale_transform matrices outside of the
    // loop so that we can save a bunch of CPU later.
    let scale_transform_matrices = (0..160)
        .map(|y| {
            let theta: Num<i32, 8> = Num::new(y) / 160 / 2;
            let scale = (num!(2.1) - theta.sin()) * num!(0.5);

            let y = Num::new(y) - (theta * 2).sin() * 8;

            AffineMatrix::from_scale(vec2(num!(1) / scale, num!(-1)))
                * AffineMatrix::from_translation(-vec2(num!(WIDTH / 2), y))
        })
        .collect::<Vec<_>>();

    let mut pos = Vector2D::default();

    let mut button_controller = ButtonController::new();

    loop {
        button_controller.update();

        let mut frame = gfx.frame();

        pos += button_controller.vector();

        if button_controller.x_tri() == Tri::Zero {
            pos.x += num!(0.5);
        }

        let bg_id = bg.show(&mut frame);
        let transform_dma = bg_id.transform_dma();

        // Calculate new transforms taking into consideration the new centre point
        let transforms = scale_transform_matrices
            .iter()
            .map(|&line_matrix| {
                AffineMatrixBackground::from(AffineMatrix::from_translation(pos) * line_matrix)
            })
            .collect::<Vec<_>>();
        HBlankDma::new(transform_dma, &transforms).show(&mut frame);

        help.show(&mut frame);

        frame.commit();
    }
}

fn grid_background() -> AffineBackground {
    let mut bg = AffineBackground::new(
        Priority::P1,
        AffineBackgroundSize::Background64x64,
        AffineBackgroundWrapBehaviour::Wrap,
    );

    // Fill the background with a basic grid pattern
    for y in 0..64 {
        for x in 0..64 {
            bg.set_tile((x, y), &backgrounds::GRID.tiles, x % 2 + (y % 2) * 2);
        }
    }

    bg
}

fn help_background() -> RegularBackground {
    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        backgrounds::HELP.tiles.format(),
    );

    bg.fill_with(&backgrounds::HELP);

    bg
}
