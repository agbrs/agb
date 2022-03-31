#![no_std]
#![no_main]

use agb::display::{palette16::Palette16, tiled::TileSetting, Priority};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();
    let vblank = agb::interrupt::VBlank::get();

    vram.set_background_palettes(&[Palette16::new([
        0xff00, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    ])]);

    let mut bg = gfx.background(Priority::P0);

    for y in 0..20u16 {
        for x in 0..30u16 {
            let dynamic_tile = vram.new_dynamic_tile();

            for (i, bit) in dynamic_tile.tile_data.iter_mut().enumerate() {
                *bit = ((((x + i as u16) % 8) << 4) | ((y + i as u16) % 8)) as u8
            }

            bg.set_tile(
                &mut vram,
                (x, y).into(),
                &dynamic_tile.tile_set(),
                TileSetting::from_raw(dynamic_tile.tile_index()),
            );

            vram.remove_dynamic_tile(dynamic_tile);
        }
    }

    bg.commit();
    bg.show();

    loop {
        vblank.wait_for_vblank();
    }
}
