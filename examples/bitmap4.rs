#![no_std]
#![feature(start)]

extern crate gba;

use gba::display;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let gba = gba::Gba::new();
    let bitmap = gba.display.bitmap4();

    bitmap.set_palette_entry(1, 0x001F);
    bitmap.set_palette_entry(2, 0x03E0);

    bitmap.draw_point_page(
        display::WIDTH / 2,
        display::HEIGHT / 2,
        1,
        display::Page::Front,
    );
    bitmap.draw_point_page(
        display::WIDTH / 2 + 5,
        display::HEIGHT / 2,
        2,
        display::Page::Back,
    );

    gba::interrupt::enable(gba::interrupt::Interrupt::VBlank);
    gba::interrupt::enable_interrupts();
    gba::display::enable_VBlank_interrupt();

    let mut count = 0;

    loop {
        gba::display::wait_for_VBlank();
        count += 1;
        if count % 6 == 0 {
            bitmap.flip_page();
        }
    }
}
