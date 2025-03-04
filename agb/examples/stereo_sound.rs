#![no_std]
#![no_main]

use agb::{
    display::{
        palette16::Palette16,
        tiled::{
            DynamicTile, RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER,
        },
        Font, Priority,
    },
    include_font, include_wav,
    sound::mixer::{Frequency, SoundChannel},
    Gba,
};

use core::fmt::Write;

// Music - "Let it in" by Josh Woodward, free download at http://joshwoodward.com
static LET_IT_IN: &[u8] = include_wav!("examples/JoshWoodward-LetItIn.wav");

static FONT: Font = include_font!("examples/font/yoster.ttf", 12);

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut gfx = gba.display.graphics.get();
    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    init_background(&mut bg);

    let mut title_renderer = FONT.render_text((0u16, 3u16));
    let mut writer = title_renderer.writer(1, 0, &mut bg);

    writeln!(&mut writer, "Let it in by Josh Woodward").unwrap();

    writer.commit();

    let timer_controller = gba.timers.timers();
    let mut timer = timer_controller.timer2;
    timer.set_enabled(true);

    let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
    mixer.enable();

    let mut channel = SoundChannel::new(LET_IT_IN);
    channel.stereo();
    mixer.play_sound(channel).unwrap();

    let mut frame_counter = 0i32;
    let mut has_written_frame_time = false;

    let mut stats_renderer = FONT.render_text((0u16, 6u16));
    loop {
        let mut frame = gfx.frame();
        bg.show(&mut frame);
        vblank_provider.wait_for_vblank();
        bg.commit();
        frame.commit();

        let before_mixing_cycles = timer.value();
        mixer.frame();
        let after_mixing_cycles = timer.value();

        frame_counter = frame_counter.wrapping_add(1);

        if frame_counter % 128 == 0 && !has_written_frame_time {
            let total_cycles = after_mixing_cycles.wrapping_sub(before_mixing_cycles) as u32;

            let percent = (total_cycles * 100) / 280896;

            let mut writer = stats_renderer.writer(1, 0, &mut bg);
            writeln!(&mut writer, "{total_cycles} cycles").unwrap();
            writeln!(&mut writer, "{percent} percent").unwrap();

            writer.commit();

            has_written_frame_time = true;
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
            bg.set_tile(
                (x, y),
                &background_tile.tile_set(),
                background_tile.tile_setting(),
            );
        }
    }
}
