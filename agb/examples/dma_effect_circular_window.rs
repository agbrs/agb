#![no_std]
#![no_main]
#![feature(isqrt)]

extern crate alloc;

use agb::{
    display::{
        example_logo,
        tiled::{RegularBackgroundSize, TileFormat},
        window::WinIn,
        HEIGHT, WIDTH,
    },
    fixnum::{Num, Rect, Vector2D},
    interrupt::VBlank,
};
use alloc::{boxed::Box, vec};

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

    let dmas = gba.dma.dma();

    example_logo::display_logo(&mut map, &mut vram);

    let mut pos: Vector2D<FNum> = (10, 10).into();
    let mut velocity: Vector2D<FNum> = Vector2D::new(1.into(), 1.into());

    let vblank = VBlank::get();

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
        let x_adjustment = x_pos << 8 | x_pos;
        for (i, value) in circle_poses.iter_mut().enumerate() {
            let i = i as i32;
            if i <= y_pos || i >= y_pos + 64 {
                *value = 0;
                continue;
            }

            *value = circle[(i - y_pos) as usize - 1] + x_adjustment;
        }

        window
            .win_in(WinIn::Win0)
            .set_position(&Rect::new(pos.floor(), (64, 65).into()));
        window.commit();

        let dma_controllable = window.win_in(WinIn::Win0).horizontal_position_dma();
        let _transfer = unsafe { dmas.dma0.hblank_transfer(&dma_controllable, &circle_poses) };

        vblank.wait_for_vblank();
    }
}
