#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    display::{
        HEIGHT, WIDTH, example_logo,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
        window::WinIn,
    },
    fixnum::{Num, Rect, Vector2D},
};
use alloc::{boxed::Box, vec};

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

    let mut dmas = gba.dma.dma();

    example_logo::display_logo(&mut map);

    let mut pos: Vector2D<FNum> = (10, 10).into();
    let mut velocity: Vector2D<FNum> = Vector2D::new(1.into(), 1.into());

    let circle: Box<[_]> = (1..64i32)
        .map(|i| {
            let y = 32 - i;
            let x = (32 * 32 - y * y).isqrt();
            let x1 = 32 - x;
            let x2 = 32 + x;

            ((x1 as u16) << 8) | (x2 as u16)
        })
        .collect();

    let mut circle_poses = vec![0; 160];
    let mut circle_transfer = None;

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
        let x_adjustment = (x_pos << 8) | x_pos;
        for (i, value) in circle_poses.iter_mut().enumerate() {
            let i = i as i32;
            if i <= y_pos || i >= y_pos + 64 {
                *value = 0;
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
            .set_position(Rect::new(pos.floor(), (64, 65).into()));

        let dma_controllable = window.win_in(WinIn::Win0).horizontal_position_dma();

        frame.commit();

        drop(circle_transfer);
        circle_transfer =
            Some(unsafe { dmas.dma0.hblank_transfer(&dma_controllable, &circle_poses) });
    }
}
