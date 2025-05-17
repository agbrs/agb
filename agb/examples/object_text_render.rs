//! An example of what's possible with object text rendering. Uses tags and colour switches
//! to display some dynamic text.
#![no_std]
#![no_main]

use agb::{
    display::{
        Palette16, Rgb15,
        font::{AlignmentKind, ChangeColour, Font, Layout, ObjectTextRenderer, SetTag, UnsetTag},
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
    let player_name = "You";

    // COLOUR_1 is the default colour and refers to palette index 1 within
    // the palette being used.
    const COLOUR_1: ChangeColour = ChangeColour::new(1);
    const COLOUR_2: ChangeColour = ChangeColour::new(2);

    const START_WIGGLY_TEXT: SetTag = SetTag::new(0);
    const STOP_WIGGLY_TEXT: UnsetTag = UnsetTag::new(0);

    // Whenever a tag is set or unset, a new letter group is created. So this
    // allows us to split the individual full stops within the ellipsis into
    // separate letter groups so that they can be rendered slowly.
    const START_SLOW_TEXT: SetTag = SetTag::new(1);
    const STOP_SLOW_TEXT: UnsetTag = UnsetTag::new(1);

    let text = format!(
        "Hey, {COLOUR_2}{player_name}{COLOUR_1}!
This uses{START_SLOW_TEXT}.{START_SLOW_TEXT}.{START_SLOW_TEXT}.{STOP_SLOW_TEXT} objects.
{START_WIGGLY_TEXT}So you can control exact positions like this.{STOP_WIGGLY_TEXT}",
    );

    let mut gfx = gba.graphics.get();

    static PALETTE: Palette16 = const {
        let mut palette = [Rgb15::BLACK; 16];
        palette[1] = Rgb15::WHITE;
        palette[2] = Rgb15(0x10_7C);
        Palette16::new(palette)
    };

    let mut layout = Layout::new(&text, &FONT, AlignmentKind::Centre, 16, 200);
    let sprite_text_render = ObjectTextRenderer::new((&PALETTE).into(), Size::S16x16);

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
                    wiggly_objects.push((sprite.pos(), sprite));
                }

                // Pause briefly at sentence breaks
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
                object.set_pos(
                    *resting
                        + vec2(
                            Num::<i32, 12>::from_raw(next_i32()) % num!(1),
                            Num::from_raw(next_i32()) % num!(1),
                        )
                        .round(),
                );
            }
            object.show(&mut frame);
        }

        frame.commit();
    }
}
