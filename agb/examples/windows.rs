//! This example shows how you can use windows to selectively display different
//! backgrounds in a rectangle.
#![no_std]
#![no_main]

use agb::{
    display::{
        HEIGHT, WIDTH, WinIn,
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
    },
    fixnum::{Num, Rect, Vector2D},
    include_background_gfx,
};

include_background_gfx!(
    mod backgrounds,
    LOGO => deduplicate "examples/gfx/test_logo.aseprite",
    BEACH => deduplicate "examples/gfx/beach-background.aseprite",
);

type FNum = Num<i32, 8>;

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}

fn main(mut gba: agb::Gba) -> ! {
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    let mut gfx = gba.graphics.get();

    let logo_bg = get_logo();
    let beach_bg = get_beach();

    let mut pos: Vector2D<FNum> = (10, 10).into();
    let mut velocity: Vector2D<FNum> = Vector2D::new(1.into(), 1.into());

    loop {
        pos += velocity;

        if pos.x.floor() > WIDTH - 64 || pos.x.floor() < 0 {
            velocity.x *= -1;
        }

        if pos.y.floor() > HEIGHT - 64 || pos.y.floor() < 0 {
            velocity.y *= -1;
        }

        let mut frame = gfx.frame();
        let logo_background_id = logo_bg.show(&mut frame);
        let beach_background_id = beach_bg.show(&mut frame);

        let window = frame.windows();
        window
            .win_in(WinIn::Win0)
            .enable_background(beach_background_id)
            .set_pos(Rect::new(pos.floor(), (64, 64).into()));

        window.win_out().enable_background(logo_background_id);

        frame.commit();
    }
}

fn get_logo() -> RegularBackground {
    let mut map = RegularBackground::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    map.fill_with(&backgrounds::LOGO);

    map
}

fn get_beach() -> RegularBackground {
    let mut map = RegularBackground::new(
        agb::display::Priority::P1,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    map.fill_with(&backgrounds::BEACH);

    map
}
