#![no_std]
#![feature(start)]

extern crate gba;

use gba::display;

struct Vector2D {
    x: i32,
    y: i32,
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let gba = gba::Gba::new();
    let bitmap = gba.display.bitmap3();

    let mut input = gba::input::ButtonController::new();
    let mut pos = Vector2D {
        x: display::WIDTH / 2,
        y: display::HEIGHT / 2,
    };

    gba::interrupt::enable(gba::interrupt::Interrupt::VBlank);
    gba::interrupt::enable_interrupts();
    gba::display::enable_VBlank_interrupt();

    loop {
        gba::display::wait_for_VBlank();

        input.update();
        pos.x += input.x_tri() as i32;
        pos.y += input.y_tri() as i32;

        pos.x = pos.x.clamp(0, display::WIDTH - 1);
        pos.y = pos.y.clamp(0, display::HEIGHT - 1);
        bitmap.draw_point(pos.x, pos.y, 0x001F);
    }
}
