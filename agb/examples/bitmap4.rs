#![no_std]
#![no_main]

use agb::display::{self, HEIGHT, WIDTH};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut bitmap = gba.display.video.bitmap4();
    let vblank = agb::interrupt::VBlank::get();

    bitmap.set_palette_entry(1, 0x001F);
    bitmap.set_palette_entry(2, 0x03E0);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            bitmap.draw_point_page(x, y, 0xF, display::bitmap4::Page::Front);
            bitmap.draw_point_page(x, y, 1, display::bitmap4::Page::Front);
            bitmap.draw_point_page(x, y, 0xFF, display::bitmap4::Page::Back);
            bitmap.draw_point_page(x, y, 2, display::bitmap4::Page::Back);
        }
    }

    let mut input = agb::input::ButtonController::new();

    loop {
        vblank.wait_for_vblank();
        input.update();

        if input.is_just_pressed(agb::input::Button::A) {
            bitmap.flip_page();
        }
    }
}
