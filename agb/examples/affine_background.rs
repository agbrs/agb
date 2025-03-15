#![no_std]
#![no_main]

use agb::{
    display::{
        Priority,
        tiled::{
            AffineBackgroundSize, AffineBackgroundTiles, AffineBackgroundWrapBehaviour,
            AffineMatrixBackground, VRAM_MANAGER,
        },
    },
    fixnum::{Num, num},
    include_background_gfx,
};

include_background_gfx!(affine_tiles, "3f3f74", water_tiles => 256 "examples/water_tiles.png");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.display.graphics.get();

    let tileset = &affine_tiles::water_tiles.tiles;

    VRAM_MANAGER.set_background_palettes(affine_tiles::PALETTES);

    let mut bg = AffineBackgroundTiles::new(
        Priority::P0,
        AffineBackgroundSize::Background32x32,
        AffineBackgroundWrapBehaviour::NoWrap,
    );

    for y in 0..32u16 {
        for x in 0..32u16 {
            bg.set_tile((x, y), tileset, 1);
        }
    }

    let mut rotation = num!(0.);
    let rotation_increase: Num<i32, 16> = num!(0.01);

    let mut input = agb::input::ButtonController::new();

    let mut scroll_x = 0;
    let mut scroll_y = 0;

    loop {
        input.update();
        scroll_x += input.x_tri() as i16;
        scroll_y += input.y_tri() as i16;

        let scroll_pos = (scroll_x, scroll_y);

        rotation += rotation_increase;
        rotation = rotation.rem_euclid(1.into());

        let transformation = AffineMatrixBackground::from_scale_rotation_position(
            (0, 0),
            (1, 1),
            rotation,
            scroll_pos,
        );

        bg.set_transform(transformation);

        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.commit();
    }
}
