#![no_std]
#![no_main]

use agb::{
    display::{
        tiled::{AffineBackgroundSize, TileFormat, TileSet, TiledMap},
        Priority,
    },
    fixnum::{num, Num},
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

    let mut rotation: Num<u16, 8> = num!(0.);
    let rotation_increase = num!(1.);

    let mut input = agb::input::ButtonController::new();

    let mut scroll_x = 0;
    let mut scroll_y = 0;

    loop {
        input.update();

        match input.x_tri() {
            Tri::Positive => scroll_x += 1,
            Tri::Negative => scroll_x -= 1,
            _ => {}
        }

        match input.y_tri() {
            Tri::Positive => scroll_y += 1,
            Tri::Negative => scroll_y -= 1,
            _ => {}
        }

        let scroll_pos = (scroll_x as i16, scroll_y as i16);
        bg.set_scroll_pos(scroll_pos.into());
        bg.set_transform((0, 0), (1, 1), 0);

        rotation += rotation_increase;
        if rotation >= num!(255.) {
            rotation = 0.into();
        }

        vblank.wait_for_vblank();
        bg.commit(&mut vram);
    }
}
