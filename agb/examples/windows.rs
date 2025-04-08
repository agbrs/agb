#![no_std]
#![no_main]

use agb::display::tiled::{RegularBackgroundTiles, TileFormat};
use agb::display::{BlendLayer, HEIGHT, WIDTH};
use agb::display::{example_logo, tiled::RegularBackgroundSize, window::WinIn};
use agb::fixnum::{Num, Rect, Vector2D, num};

type FNum = Num<i32, 8>;

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}

fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.display.graphics.get();

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    example_logo::display_logo(&mut map);

    let mut pos: Vector2D<FNum> = (10, 10).into();
    let mut velocity: Vector2D<FNum> = Vector2D::new(1.into(), 1.into());

    let mut blend_amount: Num<i32, 8> = num!(0.5);
    let mut blend_velocity: Num<i32, 8> = Num::new(1) / 128;

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

        let mut frame = gfx.frame();
        let background_id = map.show(&mut frame);

        let blend = frame.blend();
        blend
            .alpha()
            .set_layer_alpha(BlendLayer::Top, blend_amount.try_change_base().unwrap());
        blend
            .layer(BlendLayer::Top)
            .enable_background(background_id);
        blend.layer(BlendLayer::Bottom).enable_backdrop();

        let window = frame.windows();
        window
            .win_in(WinIn::Win0)
            .set_background_enable(background_id, true)
            .set_position(&Rect::new(pos.floor(), (64, 64).into()));

        window
            .win_out()
            .set_background_enable(background_id, true)
            .set_blend_enable(true);

        frame.commit();
    }
}
