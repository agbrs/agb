#![no_std]
#![no_main]

use agb::{
    display::{
        Palette16,
        font::{AlignmentKind, ChangeColour, Font, Layout, SetTag, SpriteTextRenderer, UnsetTag},
        object::Size,
    },
    fixnum::{Num, num, vec2},
    include_font,
    rng::next_i32,
};

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

    const COLOUR_1: ChangeColour = ChangeColour::new(1);
    const COLOUR_2: ChangeColour = ChangeColour::new(2);

    const SET_TAG_0: SetTag = SetTag::new(0);
    const UNSET_TAG_0: UnsetTag = UnsetTag::new(0);

    const SET_TAG_1: SetTag = SetTag::new(1);
    const UNSET_TAG_1: UnsetTag = UnsetTag::new(1);

    let text = format!(
        "Hey, {COLOUR_2}{player_name}{COLOUR_1}!\nI have a{SET_TAG_1}.{SET_TAG_1}.{SET_TAG_1}.{UNSET_TAG_1} secret.\n{SET_TAG_0}I'm so very scared of what might happen.{UNSET_TAG_0}",
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
    let mut wiggly_objects = Vec::new();
    let mut frame_count = 0;
    let mut delay: u32 = 0;

    loop {
        let mut frame = gfx.frame();
        frame_count += 1;

        if delay == 0 {
            if let Some(group) = layout.next() {
                let sprite = sprite_text_render.show(&group, vec2(16, 16));
                if group.tag() & 0b1 == 0 {
                    objects.push(sprite);
                } else {
                    wiggly_objects.push((sprite.position(), sprite));
                }

                if group.text().ends_with([',', '.', '!', '?', '\n']) {
                    delay = 16;
                } else {
                    delay = 4;
                }

                if group.tag() & 0b10 != 0 {
                    delay *= 2;
                }
            }
        } else {
            delay -= 1;
        }

        for object in objects.iter() {
            object.show(&mut frame);
        }

        for (resting, object) in wiggly_objects.iter_mut() {
            if frame_count % 4 == 0 {
                #[expect(
                    clippy::modulo_one,
                    reason = "This isn't always 0, the number is fixed point"
                )]
                object.set_position(
                    *resting
                        + (vec2(
                            Num::<i32, 12>::from_raw(next_i32()) % 1,
                            Num::from_raw(next_i32()) % 1,
                        ) + vec2(num!(0.5), num!(0.5)))
                        .floor(),
                );
            }
            object.show(&mut frame);
        }

        frame.commit();
    }
}
