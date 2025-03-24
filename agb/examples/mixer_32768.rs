#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    Gba,
    display::{
        Palette16, Priority, WIDTH,
        font::{AlignmentKind, Font, Layout, RegularBackgroundTextRenderer},
        tiled::{
            DynamicTile, RegularBackgroundSize, RegularBackgroundTiles, TileEffect, TileFormat,
            VRAM_MANAGER,
        },
    },
    include_font, include_wav,
    sound::mixer::{Frequency, SoundChannel},
};

use alloc::format;

// Music - "Crazy glue" by Josh Woodward, free download at http://joshwoodward.com
static CRAZY_GLUE: &[u8] = include_wav!("examples/JoshWoodward-CrazyGlue.wav");

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let mut gfx = gba.display.graphics.get();
    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    init_background(&mut bg);

    let timer_controller = gba.timers.timers();
    let mut timer = timer_controller.timer2;
    let mut timer2 = timer_controller.timer3;
    timer.set_enabled(true);
    timer2.set_cascade(true).set_enabled(true);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let mut channel = SoundChannel::new(CRAZY_GLUE);
    channel.stereo();
    mixer.play_sound(channel).unwrap();

    let mut frame_counter = 0i32;

    let mut text_layout = None;
    let mut renderer = RegularBackgroundTextRenderer::new((0, FONT.line_height() * 3));

    loop {
        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();

        let before_mixing_cycles_high = timer2.value();
        let before_mixing_cycles_low = timer.value();

        mixer.frame();

        let after_mixing_cycles_low = timer.value();
        let after_mixing_cycles_high = timer2.value();

        frame_counter = frame_counter.wrapping_add(1);

        if text_layout.is_none() && frame_counter % 128 == 0 {
            let before_mixing_cycles =
                ((before_mixing_cycles_high as u32) << 16) + before_mixing_cycles_low as u32;
            let after_mixing_cycles =
                ((after_mixing_cycles_high as u32) << 16) + after_mixing_cycles_low as u32;
            let total_cycles = after_mixing_cycles.wrapping_sub(before_mixing_cycles);

            let percent = (total_cycles * 100) / 280896;

            let text = format!("Mixing time: {total_cycles} cycles ({percent}%)");

            text_layout = Some(Layout::new(&text, &FONT, AlignmentKind::Left, 16, WIDTH));
        } else if let Some(text_layout) = text_layout.as_mut() {
            if let Some(lg) = text_layout.next() {
                renderer.show(&mut bg, &lg);
            }
        }
    }
}

fn init_background(bg: &mut RegularBackgroundTiles) {
    let background_tile = DynamicTile::new().fill_with(0);

    VRAM_MANAGER.set_background_palette(
        0,
        &Palette16::new([
            0x0000, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ]),
    );

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile_dynamic((x, y), &background_tile, TileEffect::default());
        }
    }

    let text_layout = Layout::new(
        "Crazy glue by Josh Woodward\njoshwoodward.com",
        &FONT,
        AlignmentKind::Centre,
        WIDTH,
        WIDTH,
    );

    let mut renderer = RegularBackgroundTextRenderer::new((0, 0));
    for lg in text_layout {
        renderer.show(bg, &lg);
    }
}
