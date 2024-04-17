#![no_std]
#![no_main]

use agb::{
    display::{
        object::{
            ChangeColour, LeftAlignLayout, ObjectUnmanaged, PaletteVram, SimpleTextRender, Size,
        },
        palette16::Palette16,
        Font, HEIGHT, WIDTH,
    },
    include_font,
    input::Button,
};
use agb_fixnum::Vector2D;

use alloc::borrow::Cow;
use core::num::NonZeroU32;

extern crate alloc;

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    let (mut unmanaged, _sprites) = gba.display.object.get_unmanaged();

    let mut palette = [0x0; 16];
    palette[1] = 0xFF_FF;
    palette[2] = 0x00_FF;
    let palette = Palette16::new(palette);
    let palette = PaletteVram::new(&palette).unwrap();

    let timer = gba.timers.timers();
    let mut timer: agb::timer::Timer = timer.timer2;

    timer.set_enabled(true);
    timer.set_divider(agb::timer::Divider::Divider256);
    let player_name = "You";
    let text = alloc::format!(
        "Woah!{change2} {player_name}! {change1}こんにちは! I have a bunch of text I want to show you... However, you will find that the amount of text I can display is limited. Who'd have thought! Good thing that my text system supports scrolling! It only took around 20 jank versions to get here!\n",
        change2 = ChangeColour::new(2),
        change1 = ChangeColour::new(1),
    );

    let start = timer.value();
    let simple = SimpleTextRender::new(
        Cow::Owned(text),
        &FONT,
        palette,
        Size::S16x16,
        Some(|c| c == '.'),
    );
    let mut wr = LeftAlignLayout::new(simple, NonZeroU32::new(WIDTH as u32));
    let end = timer.value();

    agb::println!(
        "Write took {} cycles",
        256 * (end.wrapping_sub(start) as u32)
    );

    let vblank = agb::interrupt::VBlank::get();
    let mut input = agb::input::ButtonController::new();

    let mut frame = 0;
    let mut groups_to_show = 0;

    loop {
        vblank.wait_for_vblank();
        input.update();
        let oam = &mut unmanaged.iter();

        wr.at_least_n_letter_groups(groups_to_show + 2);
        let start = timer.value();

        let can_pop_line = {
            let mut letters = wr.layout();
            let displayed_letters = letters.by_ref().take(groups_to_show);

            for (letter, slot) in displayed_letters.zip(oam) {
                let mut obj = ObjectUnmanaged::new(letter.sprite().clone());
                obj.show();
                let y = HEIGHT - 40 + letter.line() as i32 * FONT.line_height();
                obj.set_position(Vector2D::new(letter.x(), y));

                slot.set(&obj);
            }

            let speed_up = if input.is_pressed(Button::A | Button::B) {
                4
            } else {
                1
            };

            if let Some(next_letter) = letters.next() {
                if next_letter.line() < 2 {
                    if next_letter.string() == "." {
                        if frame % (16 / speed_up) == 0 {
                            groups_to_show += 1;
                        }
                    } else if frame % (4 / speed_up) == 0 {
                        groups_to_show += 1;
                    }
                    false
                } else {
                    true
                }
            } else {
                false
            }
        };

        let end = timer.value();
        agb::println!(
            "Layout took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );

        if can_pop_line && input.is_just_pressed(Button::A) {
            groups_to_show -= wr.pop_line();
        }

        frame += 1;
    }
}
