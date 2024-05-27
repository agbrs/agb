#![no_std]
#![no_main]

use agb::display::blend::{BlendMode, Layer};
use agb::display::tiled::{RegularBackgroundTiles, TileFormat};
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
    let (mut gfx, mut vram) = gba.display.video.tiled();

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut window = gba.display.window.get();

    example_logo::display_logo(&mut map, &mut vram);
    map.commit();

    let mut blend = gba.display.blend.get();

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

        let mut bg_iter = gfx.iter();
        let background_id = map.show(&mut bg_iter);

        vblank.wait_for_vblank();

        blend
            .reset()
            .set_background_enable(Layer::Top, background_id, true)
            .set_backdrop_enable(Layer::Bottom, true)
            .set_blend_mode(BlendMode::Normal)
            .set_blend_weight(Layer::Top, blend_amount.try_change_base().unwrap());

        window
            .win_in(WinIn::Win0)
            .reset()
            .set_background_enable(background_id, true)
            .set_position(&Rect::new(pos.floor(), (64, 64).into()))
            .enable();

        window
            .win_out()
            .reset()
            .enable()
            .set_background_enable(background_id, true)
            .set_blend_enable(true);

        bg_iter.commit(&mut vram);
        window.commit();
        blend.commit();
    }
}
