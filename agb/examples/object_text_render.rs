#![no_std]
#![no_main]

use agb::{
    display::{
        object::{ChangeColour, ObjectTextRender, PaletteVram, Size, TextAlignment},
        palette16::Palette16,
        Font, HEIGHT, WIDTH,
    },
    include_font,
    input::Button,
};

extern crate alloc;

use core::fmt::Write;

const FONT: Font = include_font!("examples/font/yoster.ttf", 12);
#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    let (mut unmanaged, _sprites) = gba.display.object.get_unmanaged();

    loop {
        let mut palette = [0x0; 16];
        palette[1] = 0xFF_FF;
        palette[2] = 0x00_FF;
        let palette = Palette16::new(palette);
        let palette = PaletteVram::new(&palette).unwrap();

        let timer = gba.timers.timers();
        let mut timer: agb::timer::Timer = timer.timer2;

        timer.set_enabled(true);
        timer.set_divider(agb::timer::Divider::Divider256);

        let mut wr = ObjectTextRender::new(&FONT, Size::S16x16, palette);
        let start = timer.value();

        let player_name = "You";
        let _ = writeln!(
            wr,
            "Woah!{change2} {player_name}! {change1}Hey there! I have a bunch of text I want to show you. However, you will find that the amount of text I can display is limited. Who'd have thought! Good thing that my text system supports scrolling! It only took around 20 jank versions to get here!",
            change2 = ChangeColour::new(2),
            change1 = ChangeColour::new(1),
        );
        let end = timer.value();

        agb::println!(
            "Write took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );

        let vblank = agb::interrupt::VBlank::get();
        let mut input = agb::input::ButtonController::new();

        let start = timer.value();

        wr.layout((WIDTH, 40).into(), TextAlignment::Justify, 2);
        let end = timer.value();

        agb::println!(
            "Layout took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );

        let mut line_done = false;
        let mut frame = 0;

        loop {
            vblank.wait_for_vblank();
            input.update();
            let oam = &mut unmanaged.iter();
            wr.commit(oam);

            let start = timer.value();
            if frame % 4 == 0 {
                line_done = !wr.next_letter_group();
            }
            if line_done && input.is_just_pressed(Button::A) {
                line_done = false;
                wr.pop_line();
            }
            wr.update((0, HEIGHT - 40).into());
            let end = timer.value();

            frame += 1;

            // agb::println!(
            //     "Took {} cycles, line done {}",
            //     256 * (end.wrapping_sub(start) as u32),
            //     line_done
            // );
        }
    }
}
