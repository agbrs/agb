//! Renders text onto a background.
#![no_std]
#![no_main]

use agb::{
    display::{
        Priority, Rgb15,
        font::{AlignmentKind, Font, Layout, RegularBackgroundTextRenderer},
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
    },
    include_font,
};

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palette_colour(0, 1, Rgb15::WHITE);

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut text_renderer = RegularBackgroundTextRenderer::new((4, 0));

    let mut text_layout = Layout::new(
        "Hello, World! こんにちは世界\nThis is an example of rendering text using backgrounds.",
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
