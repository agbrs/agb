#![no_std]
#![feature(start)]

extern crate agb;

use agb::display;

struct Vector2D {
    x: i32,
    y: i32,
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gba = agb::Gba::new();
    let mut bitmap = gba.display.video.bitmap3();
    let vblank = gba.display.vblank.get();

    let mut input = agb::input::ButtonController::new();
    let mut pos = Vector2D {
        x: display::WIDTH / 2,
        y: display::HEIGHT / 2,
    };

    loop {
        vblank.wait_for_VBlank();

        input.update();
        pos.x += input.x_tri() as i32;
        pos.y += input.y_tri() as i32;

        pos.x = pos.x.clamp(0, display::WIDTH - 1);
        pos.y = pos.y.clamp(0, display::HEIGHT - 1);
        bitmap.draw_point(pos.x, pos.y, 0x001F);
    }
}
