//! Text rendering onto Objects where the work is divided over multiple frames
#![no_std]
#![no_main]

use agb::{
    display::{
        Palette16, Rgb15,
        font::{AlignmentKind, Font, Layout, ObjectTextRenderer},
        object::Size,
    },
    fixnum::vec2,
    include_font,
};

use alloc::vec::Vec;

extern crate alloc;

static PALETTE: &Palette16 = const {
    let mut palette = [Rgb15::BLACK; 16];
    palette[1] = Rgb15::WHITE;
    palette[2] = Rgb15(0x10_7C);
    &Palette16::new(palette)
};

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    // use the standard graphics system
    let mut gfx = gba.graphics.get();

    // this is now mutable as we will be calling `next` on it
    let mut text_layout = Layout::new(
        "Hello, this is some text that I want to display!",
        &FONT,
        AlignmentKind::Left,
        16,
        200,
    );

    let text_render = ObjectTextRenderer::new(PALETTE.into(), Size::S16x16);
    let mut objects = Vec::new();

    loop {
        // each frame try to grab a letter group and add it to the objects list
        if let Some(letter) = text_layout.next() {
            objects.push(text_render.show(&letter, vec2(16, 16)));
        }

        let mut frame = gfx.frame();

        // render everything in the objects list
        for object in objects.iter() {
            object.show(&mut frame);
        }

        frame.commit();
    }
}
