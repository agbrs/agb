#![no_std]
#![no_main]

use agb::{
    display::{
        Palette16, Priority, Rgb15,
        font::{AlignmentKind, Font, Layout, RegularBackgroundTextRenderer},
        tiled::{
            DynamicTile16, RegularBackgroundSize, RegularBackgroundTiles, TileEffect, TileFormat,
            VRAM_MANAGER,
        },
    },
    include_font,
};

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palette(
        0,
        &Palette16::new(
            [
                0x0000, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000,
                0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            ]
            .map(Rgb15::new),
        ),
    );

    let background_tile = DynamicTile16::new().fill_with(0);

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

    let mut text_renderer = RegularBackgroundTextRenderer::new((4, 0));

    let mut text_layout = Layout::new(
        "Hello, World! こんにちは世界\nThis is a font rendering example\nHello, World! こんにちは世界\nThis is a font rendering example\nHello, World! こんにちは世界\nThis is a font rendering example\nHello, World! こんにちは世界\nThis is a font rendering example",
        &FONT,
        AlignmentKind::Left,
        32,
        200,
    );

    let mut frame = gfx.frame();
    bg.show(&mut frame);

    frame.commit();

    loop {
        if let Some(letter_group) = text_layout.next() {
            text_renderer.show(&mut bg, &letter_group);
        }

        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();
    }
}
