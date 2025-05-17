//! Mostly a port of the [tonc mode7](https://gbadev.net/tonc/mode7.html) example.
//! Shows a 3d plane that you can move around on. The maths and full explanation of how
//! it works can be found in the link above.
//!
//! This is a prime example of using dma with affine background transformations.
#![no_main]
#![no_std]
extern crate alloc;

use agb::{
    display::{
        Priority,
        tiled::{
            AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour,
            AffineMatrixBackground, RegularBackground, RegularBackgroundSize, VRAM_MANAGER,
        },
    },
    dma::HBlankDma,
    fixnum::{Num, Vector2D, num, vec2},
    include_background_gfx,
    input::{Button, ButtonController, Tri},
};
use alloc::vec::Vec;

include_background_gfx!(mod backgrounds,
    "000000",
    NUMBERS => 256 "examples/gfx/number-background.aseprite",
    HELP => deduplicate "examples/gfx/3d-plane-help-text.aseprite",
);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    let mut gfx = gba.graphics.get();

    let mut wrap_behaviour = AffineBackgroundWrapBehaviour::NoWrap;

    let mut bg = AffineBackground::new(
        Priority::P1,
        AffineBackgroundSize::Background32x32,
        wrap_behaviour,
    );

    for y in 0..32 {
        for x in 0..32 {
            bg.set_tile((x, y), &backgrounds::NUMBERS.tiles, y / 4);
        }
    }

    let mut help_bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        backgrounds::HELP.tiles.format(),
    );
    help_bg.fill_with(&backgrounds::HELP);

    let mut viewing_angle: Num<i32, 8> = num!(0);
    let mut position: Vector2D<Num<i32, 12>> = vec2(num!(16 * 8), num!(16 * 8));
    let mut height: Num<i32, 12> = num!(10);

    let mut input = ButtonController::new();

    let inverse_lookup = (0..160)
        .map(|y| num!(1) / Num::new(y + 1))
        .collect::<Vec<_>>();

    loop {
        input.update();

        if input.is_just_pressed(Button::START) {
            wrap_behaviour = match wrap_behaviour {
                AffineBackgroundWrapBehaviour::NoWrap => AffineBackgroundWrapBehaviour::Wrap,
                AffineBackgroundWrapBehaviour::Wrap => AffineBackgroundWrapBehaviour::NoWrap,
            };
        }

        viewing_angle += Num::new(input.x_tri() as i32) / 64;

        let direction: Vector2D<Num<i32, 8>> =
            vec2(input.lr_tri() as i32, input.y_tri() as i32).change_base();

        let cos_viewing_angle = viewing_angle.cos();
        let sin_viewing_angle = viewing_angle.sin();

        position += vec2(
            (cos_viewing_angle * direction.x - sin_viewing_angle * direction.y).change_base(),
            (sin_viewing_angle * direction.x + cos_viewing_angle * direction.y).change_base(),
        );

        let ab_tri = Tri::from((input.is_pressed(Button::B), input.is_pressed(Button::A)));
        height = (height + Num::new(ab_tri as i32) / 4).clamp(num!(0.1), num!(15));

        let mut frame = gfx.frame();
        bg.set_wrap_behaviour(wrap_behaviour);
        let bg_id = bg.show(&mut frame);

        let transforms = (0..160)
            .map(|y| {
                let lambda = height * inverse_lookup[y];
                let lcf = lambda * cos_viewing_angle.change_base();
                let lsf = lambda * sin_viewing_angle.change_base();

                // the order of changing base here is important. See the tonc mode7 page
                let horizontal_offset = {
                    let lxr = lcf.change_base() * 120;
                    let lyr = (lsf * 160).change_base();
                    position.x.change_base::<i32, 8>() - lxr + lyr
                };
                let vertical_offset = {
                    let lxr = lsf.change_base() * 120;
                    let lyr = (lcf * 160).change_base();
                    position.y.change_base::<i32, 8>() - lxr - lyr
                };

                AffineMatrixBackground {
                    a: lcf.try_change_base().unwrap(),
                    b: num!(0),
                    c: lsf.try_change_base().unwrap(),
                    d: num!(1),
                    x: horizontal_offset.change_base(),
                    y: vertical_offset.change_base(),
                }
            })
            .collect::<Vec<_>>();

        HBlankDma::new(bg_id.transform_dma(), &transforms).show(&mut frame);

        help_bg.show(&mut frame);
        frame.commit();
    }
}
