#![no_std]
#![no_main]

use agb::{
    display::{
        object::{
            font::{Configuration, WordRender},
            PaletteVram, Size,
        },
        palette16::Palette16,
        Font,
    },
    fixnum::num,
    include_font,
    timer::Divider,
};
use agb_fixnum::Num;

use core::fmt::Write;

const FONT: Font = include_font!("examples/font/yoster.ttf", 12);
#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    let (mut unmanaged, mut sprites) = gba.display.object.get_unmanaged();

    let mut palette = [0x0; 16];
    palette[1] = 0xFF_FF;
    let palette = Palette16::new(palette);
    let palette = PaletteVram::new(&palette).unwrap();

    let config = Configuration::new(Size::S32x16, palette);

    let mut wr = WordRender::new(&FONT, config);

    let mut number: Num<i32, 8> = num!(1.25235);

    let vblank = agb::interrupt::VBlank::get();
    let mut input = agb::input::ButtonController::new();

    let timer = gba.timers.timers();
    let mut timer = timer.timer2;

    timer.set_enabled(true);
    timer.set_divider(agb::timer::Divider::Divider64);

    loop {
        vblank.wait_for_vblank();
        input.update();

        number += num!(0.01) * input.y_tri() as i32;

        let start = timer.value();

        let _ = writeln!(wr, "abcdefgh ijklmnopq rstuvwxyz");
        let line = wr.get_line();
        let rasterised = timer.value();

        let oam_frmae = &mut unmanaged.iter();
        line.unwrap().draw(oam_frmae);
        let drawn = timer.value();

        let start_to_end = to_ms(drawn.wrapping_sub(start));
        let raster = to_ms(rasterised.wrapping_sub(start));
        let object = to_ms(drawn.wrapping_sub(rasterised));

        agb::println!("Start: {start_to_end:.3}");
        agb::println!("Raster: {raster:.3}");
        agb::println!("Object: {object:.3}");
    }
}

fn to_ms(time: u16) -> Num<i32, 8> {
    Num::new(time as i32) * num!(3.815) / 1000
}
