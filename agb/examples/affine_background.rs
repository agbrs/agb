#![no_std]
#![no_main]

use agb::{
    display::{
        tiled::{AffineBackgroundSize, TileFormat, TileSet, TiledMap},
        Priority,
    },
    fixnum::num,
    include_gfx,
    input::Tri,
};

include_gfx!("examples/affine_tiles.toml");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled2();
    let vblank = agb::interrupt::VBlank::get();

    let tileset = TileSet::new(affine_tiles::water_tiles.tiles, TileFormat::EightBpp);

    vram.set_background_palettes(affine_tiles::water_tiles.palettes);

    let mut bg = gfx.background(Priority::P0, AffineBackgroundSize::Background32x32);

    for y in 0..32u16 {
        for x in 0..32u16 {
            bg.set_tile(&mut vram, (x, y).into(), &tileset, 1);
        }
    }

    bg.commit(&mut vram);
    bg.show();

    let mut rotation = num!(0.);
    let rotation_increase = num!(1.);

    let mut input = agb::input::ButtonController::new();

    loop {
        input.update();

        let x_dir = match input.x_tri() {
            Tri::Positive => (1, 0).into(),
            Tri::Negative => (-1, 0).into(),
            _ => (0, 0).into(),
        };

        let y_dir = match input.y_tri() {
            Tri::Positive => (0, 1).into(),
            Tri::Negative => (0, -1).into(),
            _ => (0, 0).into(),
        };

        let new_scroll_pos = bg.scroll_pos() + x_dir + y_dir;
        bg.set_scroll_pos(new_scroll_pos);
        bg.set_transform((0i16, 0i16).into(), (1, 1).into(), rotation);

        rotation += rotation_increase;
        if rotation >= num!(255.) {
            rotation = 0.into();
        }

        vblank.wait_for_vblank();
        bg.commit(&mut vram);
    }
}
