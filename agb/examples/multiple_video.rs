#![no_std]
#![no_main]

extern crate agb;

use agb::display;

struct Vector2D {
    x: i32,
    y: i32,
}

#[no_mangle]
pub fn main() -> ! {
    let mut gba = agb::Gba::new();
    let vblank = agb::interrupt::VBlank::new();
    let mut input = agb::input::ButtonController::new();

    loop {
        bitmap3_mode(&mut gba.display.video.bitmap3(), &vblank, &mut input);
        bitmap4_mode(&mut gba.display.video.bitmap4(), &vblank, &mut input);
    }
}

fn bitmap3_mode(
    bitmap: &mut display::bitmap3::Bitmap3,
    vblank: &agb::interrupt::VBlank,
    input: &mut agb::input::ButtonController,
) {
    let mut pos = Vector2D {
        x: display::WIDTH / 2,
        y: display::HEIGHT / 2,
    };

    loop {
        vblank.wait_for_vblank();

        input.update();
        if input.is_just_pressed(agb::input::Button::B) {
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
    vblank: &agb::interrupt::VBlank,
    input: &mut agb::input::ButtonController,
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
        vblank.wait_for_vblank();

        input.update();
        if input.is_just_pressed(agb::input::Button::B) {
            break;
        }

        count += 1;
        if count % 6 == 0 {
            bitmap.flip_page();
        }
    }
}
