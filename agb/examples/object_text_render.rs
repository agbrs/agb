#![no_std]
#![no_main]

use agb::{
    display::{
        object::{
            font::{ObjectTextRender, TextAlignment},
            PaletteVram, Size,
        },
        palette16::Palette16,
        Font, WIDTH,
    },
    include_font,
    input::Button,
};
use agb_fixnum::Rect;

extern crate alloc;
use alloc::vec::Vec;

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

        let mut wr = ObjectTextRender::new(&FONT, Size::S16x8, palette);
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

        wr.set_alignment(TextAlignment::Left);
        wr.set_size((WIDTH / 3, 20).into());
        wr.set_paragraph_spacing(2);
        wr.layout();

        loop {
            vblank.wait_for_vblank();
            input.update();
            let oam = &mut unmanaged.iter();
            wr.commit(oam, (WIDTH / 3, 0).into());

            let start = timer.value();
            let line_done = !wr.next_letter_group();
            if line_done && input.is_just_pressed(Button::A) {
                wr.pop_line();
            }
            wr.layout();
            let end = timer.value();

            agb::println!(
                "Took {} cycles, line done {}",
                256 * (end.wrapping_sub(start) as u32),
                line_done
            );
        }
    }
}
