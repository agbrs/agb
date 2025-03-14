#![no_std]
#![no_main]

use agb::{
    display::{
        Font, Priority,
        palette16::Palette16,
        tiled::{
            DynamicTile, RegularBackgroundSize, RegularBackgroundTiles, TileEffect, TileFormat,
            VRAM_MANAGER,
        },
    },
    include_font,
};

use core::fmt::Write;

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.display.graphics.get();

    VRAM_MANAGER.set_background_palette(
        0,
        &Palette16::new([
            0x0000, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ]),
    );

    let background_tile = DynamicTile::new().fill_with(0);

    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile_dynamic((x, y), &background_tile, TileEffect::default());
        }
    }

    let mut renderer = FONT.render_text((0u16, 3u16));
    let mut writer = renderer.writer(1, 2, &mut bg);

    writeln!(&mut writer, "Hello, World! こんにちは世界").unwrap();
    writeln!(&mut writer, "This is a font rendering example").unwrap();

    writer.commit();

    let mut frame = gfx.frame();
    bg.show(&mut frame);

    frame.commit();

    let mut frame_count = 0;

    loop {
        let mut frame = gfx.frame();

        let mut renderer = FONT.render_text((4u16, 0u16));
        let mut writer = renderer.writer(1, 2, &mut bg);

        writeln!(&mut writer, "Frame {frame_count}").unwrap();
        writer.commit();

        frame_count += 1;

        bg.show(&mut frame);

        frame.commit();
    }
}
