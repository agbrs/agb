#![no_std]
#![no_main]

use agb::display::{
    Palette16, Priority, Rgb15,
    tiled::{
        DynamicTile16, RegularBackgroundSize, RegularBackgroundTiles, TileEffect, TileFormat,
        VRAM_MANAGER,
    },
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(&[Palette16::new(
        [
            0xff00, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ]
        .map(Rgb15::new),
    )]);

    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for y in 0..20u32 {
        for x in 0..30u32 {
            let dynamic_tile = DynamicTile16::new();

            for (i, bit) in dynamic_tile.tile_data.iter_mut().enumerate() {
                let i = i as u32;
                let mut value = 0;

                for j in 0..4 {
                    value |= (value << 8) | ((x + i) % 8) | (((y + j) % 8) << 4);
                }

                *bit = value;
            }

            bg.set_tile_dynamic((x as u16, y as u16), &dynamic_tile, TileEffect::default());
        }
    }

    loop {
        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.commit();
    }
}
