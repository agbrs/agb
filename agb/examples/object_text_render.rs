#![no_std]
#![no_main]

use agb::{
    display::{
        object::{
            font::{BufferedWordRender, Configuration},
            PaletteVram, Size,
        },
        palette16::Palette16,
        Font,
    },
    include_font,
    input::Button,
};

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
        let palette = Palette16::new(palette);
        let palette = PaletteVram::new(&palette).unwrap();

        let config = Configuration::new(Size::S32x16, palette);

        let mut wr = BufferedWordRender::new(&FONT, config);
        let _ = writeln!(
        wr,
        "Hello there!\nI spent this weekend\nwriting this text system!\nIs it any good?\n\nOh, by the way, you can\npress A to restart!"
    );

        let vblank = agb::interrupt::VBlank::get();
        let mut input = agb::input::ButtonController::new();

        let timer = gba.timers.timers();
        let mut timer: agb::timer::Timer = timer.timer2;

        timer.set_enabled(true);
        timer.set_divider(agb::timer::Divider::Divider64);

        let mut num_letters = 0;
        let mut frame = 0;

        loop {
            vblank.wait_for_vblank();
            input.update();
            let oam_frmae = &mut unmanaged.iter();

            let start = timer.value();
            wr.draw_partial(oam_frmae, (0, 0).into(), num_letters);
            let end = timer.value();

            agb::println!("Took {} cycles", 64 * (end.wrapping_sub(start) as u32));

            frame += 1;

            if frame % 4 == 0 {
                num_letters += 1;
            }
            wr.process();

            if input.is_just_pressed(Button::A) {
                break;
            }
        }
    }
}
