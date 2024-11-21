#![no_std]
#![no_main]

use agb::{
    display::{
        object::{Alignment, MultiLineTextDisplay, ObjectUnmanaged, PaletteVram, Size, TextBlock},
        palette16::Palette16,
        Font, HEIGHT, WIDTH,
    },
    include_font,
    input::Button,
};

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

    let start = timer.value();
    let wr = TextBlock::new(
        &FONT,
        "Woah!{change2} {player_name}!{change1} こんにちは!\n\nI have a bunch of text I want to show you. However, you will find that the amount of text I can display is limited. Who'd have thought! Good thing that my text system supports scrolling! It only took around 20 jank versions to get here!",
        palette,
        Alignment::Left,
        (WIDTH - 8) as u32,
        Size::S16x16,
    );

    let end = timer.value();

    agb::println!(
        "Write took {} cycles",
        256 * (end.wrapping_sub(start) as u32)
    );

    let vblank = agb::interrupt::VBlank::get();
    let mut input = agb::input::ButtonController::new();

    let mut frame = 0;

    let mut multi_line = MultiLineTextDisplay::new(wr, 2);

    loop {
        input.update();

        let start = timer.value();

        multi_line.do_work();

        if frame > 16 && frame % 4 == 0 {
            multi_line.increase_letters();
        }

        if frame > 16
            && multi_line.is_showing_all_available_lines()
            && input.is_just_pressed(Button::A)
        {
            multi_line.pop_line();
        }

        let end = timer.value();
        agb::println!(
            "Update took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );
        let start = timer.value();

        let mut frame_oam = unmanaged.iter();

        for letter in multi_line.iter() {
            let mut object = ObjectUnmanaged::new(letter.letter);
            object
                .set_position((4 + letter.x, HEIGHT - 32 + letter.line * 16).into())
                .show();
            frame_oam.set_next(&object);
        }

        let end = timer.value();
        agb::println!(
            "Draw took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );

        vblank.wait_for_vblank();
        frame += 1;
    }
}
