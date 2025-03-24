#![no_std]
#![no_main]

use agb::{
    display::{
        font::{AlignmentKind, ChangeColour, Font, Layout, SpriteTextRenderer},
        object::Size,
        palette16::Palette16,
    },
    include_font,
};
use agb_fixnum::vec2;
use alloc::{format, vec::Vec};

extern crate alloc;

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    let timer = gba.timers.timers();
    let mut timer: agb::timer::Timer = timer.timer2;

    timer.set_enabled(true);
    timer.set_divider(agb::timer::Divider::Divider256);

    let start = timer.value();
    let player_name = "You";

    let colour1 = ChangeColour::new(1);
    let colour2 = ChangeColour::new(2);

    let text = format!(
        "Woah! {colour2}{player_name}{colour1}! I have a bunch of text I want to show you. However, you will find that the amount of text I can display is limited.\nWho'd have thought? Good thing that my text system supports scrolling! It only took around 20 jank versions to get here!",
    );
    let end = timer.value();

    agb::println!(
        "Write took {} cycles",
        256 * (end.wrapping_sub(start) as u32)
    );

    let mut gfx = gba.display.graphics.get();

    static PALETTE: Palette16 = const {
        let mut palette = [0x0; 16];
        palette[1] = 0xFF_FF;
        palette[2] = 0x10_7C;
        Palette16::new(palette)
    };

    let mut layout = Layout::new(&text, &FONT, AlignmentKind::Centre, 16, 200);
    let sprite_text_render = SpriteTextRenderer::new((&PALETTE).into(), Size::S16x16);

    let mut objects = Vec::new();

    loop {
        let mut frame = gfx.frame();

        if let Some(group) = layout.next() {
            objects.push(sprite_text_render.show(&group, vec2(16, 16)));
        }

        for object in objects.iter() {
            object.show(&mut frame);
        }

        frame.commit();
    }
}
