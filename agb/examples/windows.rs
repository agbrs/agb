#![no_std]
#![no_main]

use agb::display::blend::{BlendMode, Layer};
use agb::display::tiled::TileFormat;
use agb::display::{example_logo, tiled::RegularBackgroundSize, window::WinIn};
use agb::display::{HEIGHT, WIDTH};
use agb::fixnum::{num, Num, Rect, Vector2D};
use agb::interrupt::VBlank;

type FNum = Num<i32, 8>;

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}

fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();

    let mut map = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut window = gba.display.window.get();
    window
        .win_in(WinIn::Win0)
        .set_background_enable(map.background(), true)
        .set_position(&Rect::new((10, 10).into(), (64, 64).into()))
        .enable();

    window
        .win_out()
        .enable()
        .set_background_enable(map.background(), true)
        .set_blend_enable(true);

    example_logo::display_logo(&mut map, &mut vram);

    let mut blend = gba.display.blend.get();

    blend
        .set_background_enable(Layer::Top, map.background(), true)
        .set_backdrop_enable(Layer::Bottom, true)
        .set_blend_mode(BlendMode::Normal);

    let mut pos: Vector2D<FNum> = (10, 10).into();
    let mut velocity: Vector2D<FNum> = Vector2D::new(1.into(), 1.into());

    let mut blend_amount: Num<i32, 8> = num!(0.5);
    let mut blend_velocity: Num<i32, 8> = Num::new(1) / 128;

    let vblank = VBlank::get();

    loop {
        pos += velocity;

        if pos.x.floor() > WIDTH - 64 || pos.x.floor() < 0 {
            velocity.x *= -1;
        }

        if pos.y.floor() > HEIGHT - 64 || pos.y.floor() < 0 {
            velocity.y *= -1;
        }

        blend_amount += blend_velocity;
        if blend_amount > num!(0.75) || blend_amount < num!(0.25) {
            blend_velocity *= -1;
        }

        blend_amount = blend_amount.clamp(0.into(), 1.into());

        blend.set_blend_weight(Layer::Top, blend_amount.try_change_base().unwrap());

        window
            .win_in(WinIn::Win0)
            .set_position(&Rect::new(pos.floor(), (64, 64).into()));

        vblank.wait_for_vblank();
        window.commit();
        blend.commit();
    }
}
