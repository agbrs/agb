#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    display::{
        HEIGHT, WIDTH, WinIn,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    dma::HBlankDmaDefinition,
    fixnum::{Num, Rect, Vector2D, vec2},
    include_background_gfx,
};

use alloc::{boxed::Box, vec};

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}

fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let map = get_logo();

    let mut pos: Vector2D<Num<i32, 8>> = vec2(10.into(), 10.into());
    let mut velocity = vec2(1.into(), 1.into());

    let circle: Box<[_]> = (1..64i32)
        .map(|i| {
            let y = 32 - i;
            let x = (32 * 32 - y * y).isqrt();
            let x1 = 32 - x;
            let x2 = 32 + x;

            vec2(x2 as u8, x1 as u8)
        })
        .collect();

    let mut circle_poses = vec![vec2(0, 0); 160];

    loop {
        pos += velocity;

        if pos.x.floor() > WIDTH - 64 || pos.x.floor() < 0 {
            velocity.x *= -1;
        }

        if pos.y.floor() > HEIGHT - 64 || pos.y.floor() < 0 {
            velocity.y *= -1;
        }

        let x_pos = pos.x.floor().max(0) as u16;
        let y_pos = pos.y.floor().max(0);
        let x_adjustment = vec2(x_pos as u8, x_pos as u8);
        for (i, value) in circle_poses.iter_mut().enumerate() {
            let i = i as i32;
            if i <= y_pos || i >= y_pos + 64 {
                *value = vec2(0, 0);
                continue;
            }

            *value = circle[(i - y_pos) as usize - 1] + x_adjustment;
        }

        let mut frame = gfx.frame();
        let background_id = map.show(&mut frame);

        let window = frame.windows();

        window
            .win_in(WinIn::Win0)
            .enable_background(background_id)
            .set_pos(Rect::new(pos.floor(), (64, 65).into()));

        let dma_controllable = window.win_in(WinIn::Win0).horizontal_pos_dma();
        HBlankDmaDefinition::new(dma_controllable, &circle_poses).show(&mut frame);

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
