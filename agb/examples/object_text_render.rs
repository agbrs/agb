#![no_std]
#![no_main]

use agb::{
    display::{
        object::{
            font::{BufferedWordRender, Configuration},
            PaletteVram, Size,
        },
        palette16::Palette16,
        Font, WIDTH,
    },
    include_font,
    input::Button,
};
use agb_fixnum::Rect;

use core::fmt::Write;

const FONT: Font = include_font!("examples/font/pixelated.ttf", 8);
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

        let config = Configuration::new(Size::S16x8, palette);

        let mut wr = BufferedWordRender::new(&FONT, config);
        let _ = writeln!(
            wr,
            "{}",
            "counts for three shoot dice for damage calculation\nmalfunctions all dice after use"
                .to_ascii_uppercase()
        );

        let vblank = agb::interrupt::VBlank::get();
        let mut input = agb::input::ButtonController::new();

        let timer = gba.timers.timers();
        let mut timer: agb::timer::Timer = timer.timer2;

        timer.set_enabled(true);
        timer.set_divider(agb::timer::Divider::Divider256);

        let mut num_letters = 0;
        let mut frame = 0;

        loop {
            vblank.wait_for_vblank();
            input.update();
            let oam = &mut unmanaged.iter();
            wr.commit(oam);

            let start = timer.value();
            wr.update(
                Rect::new((WIDTH / 8, 0).into(), (80, 100).into()),
                num_letters,
            );
            wr.process();
            let end = timer.value();

            agb::println!("Took {} cycles", 256 * (end.wrapping_sub(start) as u32));

            frame += 1;

            // if frame % 2 == 0 {
            num_letters += 1;
            // }

            if input.is_just_pressed(Button::A) {
                break;
            }
        }
        let start = timer.value();
        drop(wr);
        let oam = unmanaged.iter();
        drop(oam);
        let end = timer.value();
        agb::println!(
            "Drop took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );
    }
}
