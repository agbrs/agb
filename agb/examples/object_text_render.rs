#![no_std]
#![no_main]

use agb::{
    display::{
        object::{
            ChangeColour, ObjectTextRender, ObjectUnmanaged, PaletteVram, Size, TextAlignment,
        },
        palette16::Palette16,
        Font, HEIGHT, WIDTH,
    },
    include_font,
    input::Button,
};
use agb_fixnum::Vector2D;

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
    let mut wr = ObjectTextRender::new(text, &FONT, Size::S16x16, palette, Some(|c| c == '.'));
    let end = timer.value();

    agb::println!(
        "Write took {} cycles",
        256 * (end.wrapping_sub(start) as u32)
    );

    let vblank = agb::interrupt::VBlank::get();
    let mut input = agb::input::ButtonController::new();

    let start = timer.value();

    wr.layout(WIDTH, TextAlignment::Justify, 2);
    let end = timer.value();

    agb::println!(
        "Layout took {} cycles",
        256 * (end.wrapping_sub(start) as u32)
    );

    let mut line = 0;
    let mut frame = 0;
    let mut groups_to_show = 0;

    loop {
        vblank.wait_for_vblank();
        input.update();
        let oam = &mut unmanaged.iter();

        let done_rendering = !wr.next_letter_group();

        let mut letters = wr.letter_groups();
        let displayed_letters = letters
            .by_ref()
            .take(groups_to_show)
            .filter(|x| x.line() >= line);

        for (letter, slot) in displayed_letters.zip(oam) {
            slot.set(&ObjectUnmanaged::from(
                &letter + Vector2D::new(0, HEIGHT - 40 - line * FONT.line_height()),
            ))
        }

        if let Some(next_letter) = letters.next() {
            if next_letter.line() < line + 2 {
                if next_letter.letters() == "." {
                    if frame % 16 == 0 {
                        groups_to_show += 1;
                    }
                } else if frame % 4 == 0 {
                    groups_to_show += 1;
                }
            } else if input.is_just_pressed(Button::A) {
                line += 1;
            }
        }

        wr.update();

        frame += 1;
    }
}
