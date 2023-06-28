#![no_std]
#![no_main]

use agb::{
    display::{
        object::{
            font::{BufferedRender, LayoutCache, TextAlignment},
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

        let mut wr = BufferedRender::new(&FONT, Size::S16x8, palette);
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

        let mut alignment = TextAlignment::Left;

        let mut cache = LayoutCache::new();

        loop {
            vblank.wait_for_vblank();
            input.update();
            let oam = &mut unmanaged.iter();
            cache.commit(oam);

            let start = timer.value();
            wr.process();
            cache.update(
                &mut wr,
                Rect::new((WIDTH / 3, 0).into(), (WIDTH / 3, 100).into()),
                alignment,
                2,
                num_letters,
            );
            let end = timer.value();

            agb::println!("Took {} cycles", 256 * (end.wrapping_sub(start) as u32));

            if input.is_just_pressed(Button::LEFT) {
                alignment = TextAlignment::Left;
            }
            if input.is_just_pressed(Button::RIGHT) {
                alignment = TextAlignment::Right;
            }
            if input.is_just_pressed(Button::UP | Button::DOWN) {
                alignment = TextAlignment::Center;
            }

            num_letters += 1;

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
