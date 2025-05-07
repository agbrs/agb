#![no_std]
#![no_main]

use agb::{
    display::{
        HEIGHT, Layer, WIDTH, WinIn,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    fixnum::{Num, Rect, Vector2D, num},
    include_background_gfx,
};

type FNum = Num<i32, 8>;

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}

fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let map = get_logo();

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
            .set_layer_alpha(Layer::Top, blend_amount.try_change_base().unwrap());
        blend.layer(Layer::Top).enable_background(background_id);
        blend.layer(Layer::Bottom).enable_backdrop();

        let window = frame.windows();
        window
            .win_in(WinIn::Win0)
            .enable_background(background_id)
            .set_pos(Rect::new(pos.floor(), (64, 64).into()));

        window
            .win_out()
            .enable_background(background_id)
            .enable_blending();

        frame.commit();
    }
}

fn get_logo() -> RegularBackgroundTiles {
    include_background_gfx!(mod backgrounds, LOGO => "examples/gfx/test_logo.aseprite");

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);
    map.fill_with(&backgrounds::LOGO);

    map
}
