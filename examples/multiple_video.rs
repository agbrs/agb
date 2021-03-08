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
    let mut gba = gba::Gba::new();
    let mut vblank = gba.display.vblank.get();
    let mut input = gba::input::ButtonController::new();

    loop {
        bitmap3_mode(&mut gba.display.video.bitmap3(), &mut vblank, &mut input);
        bitmap4_mode(&mut gba.display.video.bitmap4(), &mut vblank, &mut input);
    }
}

fn bitmap3_mode(
    bitmap: &mut display::bitmap3::Bitmap3,
    vblank: &mut display::VBlank,
    input: &mut gba::input::ButtonController,
) {
    let mut pos = Vector2D {
        x: display::WIDTH / 2,
        y: display::HEIGHT / 2,
    };

    loop {
        vblank.wait_for_VBlank();

        input.update();
        if input.is_just_pressed(gba::input::Button::B) {
            break;
        }

        pos.x += input.x_tri() as i32;
        pos.y += input.y_tri() as i32;

        pos.x = pos.x.clamp(0, display::WIDTH - 1);
        pos.y = pos.y.clamp(0, display::HEIGHT - 1);
        bitmap.draw_point(pos.x, pos.y, 0x001F);
    }
}

fn bitmap4_mode(
    bitmap: &mut display::bitmap4::Bitmap4,
    vblank: &mut display::VBlank,
    input: &mut gba::input::ButtonController,
) {
    bitmap.set_palette_entry(1, 0x001F);
    bitmap.set_palette_entry(2, 0x03E0);

    bitmap.draw_point_page(
        display::WIDTH / 2,
        display::HEIGHT / 2,
        1,
        display::bitmap4::Page::Front,
    );
    bitmap.draw_point_page(
        display::WIDTH / 2 + 5,
        display::HEIGHT / 2,
        2,
        display::bitmap4::Page::Back,
    );

    let mut count = 0;
    loop {
        vblank.wait_for_VBlank();

        input.update();
        if input.is_just_pressed(gba::input::Button::B) {
            break;
        }

        count += 1;
        if count % 6 == 0 {
            bitmap.flip_page();
        }
    }
}
