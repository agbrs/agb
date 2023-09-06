#![no_std]
#![no_main]

use agb::display::{
    palette16::Palette16,
    tiled::{RegularBackgroundSize, TileFormat, TiledMap},
    Priority,
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();
    let vblank = agb::interrupt::VBlank::get();

    vram.set_background_palettes(&[Palette16::new([
        0xff00, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    ])]);

    let mut bg = gfx.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for y in 0..20u32 {
        for x in 0..30u32 {
            let dynamic_tile = vram.new_dynamic_tile();

            for (i, bit) in dynamic_tile.tile_data.iter_mut().enumerate() {
                let i = i as u32;
                let mut value = 0;

                for j in 0..4 {
                    value |= (value << 8) | ((x + i) % 8) | ((y + j) % 8) << 4;
                }

                *bit = value;
            }

            bg.set_tile(
                &mut vram,
                (x as u16, y as u16).into(),
                &dynamic_tile.tile_set(),
                dynamic_tile.tile_setting(),
            );

            vram.remove_dynamic_tile(dynamic_tile);
        }
    }

    bg.commit(&mut vram);
    bg.show();

    loop {
        vblank.wait_for_vblank();
    }
}
