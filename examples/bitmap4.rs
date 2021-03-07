#![no_std]
#![feature(start)]

extern crate gba;

use gba::display;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let gba = gba::Gba::new();
    let bitmap = gba.display.bitmap4();
    let vblank = gba.display.get_vblank();

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
        count += 1;
        if count % 6 == 0 {
            bitmap.flip_page();
        }
    }
}
