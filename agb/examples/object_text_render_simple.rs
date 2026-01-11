//! A simple example of object text rendering that demonstrates the most simple
//! way of using it. Normally you would divide the work over a few frames, which
//! this does not do.
#![no_std]
#![no_main]

use agb::{
    display::{
        Palette16, Rgb, Rgb15,
        font::{AlignmentKind, Font, Layout, ObjectTextRenderer},
        object::Size,
        tiled::VRAM_MANAGER,
    },
    fixnum::vec2,
    include_font,
};

use alloc::vec::Vec;

extern crate alloc;

static PALETTE: &Palette16 = const {
    let mut palette = [Rgb15::BLACK; 16];
    palette[1] = Rgb15::WHITE;
    &Palette16::new(palette)
};

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb::new(0, 97, 132).into());

    let layout = Layout::new(
        "Hello, this is some text that I want to display!",
        &FONT,
        AlignmentKind::Left,
        16,
        200,
    )
    .with_dropshadow(2);
    let text_render = ObjectTextRenderer::new(PALETTE.into(), Size::S16x16);

    let objects: Vec<_> = layout.map(|x| text_render.show(&x, vec2(16, 16))).collect();

    loop {
        let mut frame = gfx.frame();

        for object in objects.iter() {
            object.show(&mut frame);
        }

        frame.commit();
    }
}
