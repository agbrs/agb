#![no_std]
#![no_main]

use agb::display::{self, bitmap4::Bitmap4};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut bitmap = gba.display.video.get::<Bitmap4>();
    let vblank = agb::interrupt::VBlank::get();

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
        count += 1;
        if count % 6 == 0 {
            bitmap.flip_page();
        }
    }
}
